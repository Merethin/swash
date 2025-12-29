use std::{error::Error, io, path::PathBuf};
use futures::TryStreamExt;
use log::warn;
use quick_xml::de::Deserializer;
use reqwest::{Client, Url};
use async_compression::tokio::bufread::GzipDecoder;
use serde::Deserialize;
use tokio_util::io::StreamReader;
use tokio::{io::{AsyncBufRead, AsyncRead, BufReader}, fs::File};
use quick_xml::events::{Event, BytesStart};
use sqlx::PgPool;

use crate::models::{Nation, Region};

const BUFFER_SIZE: usize = 32 * 1024; // 32 KiB

pub async fn stream_data_dump_from_url(pool: &PgPool, client: &Client, url: Url) -> Result<(), Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let stream = StreamReader::new(
        response.bytes_stream().map_err(io::Error::other)
    );

    stream_data_dump(pool, BufReader::with_capacity(BUFFER_SIZE, stream)).await
}

pub async fn stream_data_dump_from_local(pool: &PgPool, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let file = File::open(path).await?;

    stream_data_dump(pool, BufReader::with_capacity(BUFFER_SIZE, file)).await
}

const NATION_BATCH_SIZE: usize = 3000;
const REGION_BATCH_SIZE: usize = 1000;

async fn stream_data_dump<R: AsyncRead + Unpin>(pool: &PgPool, input: BufReader<R>) -> Result<(), Box<dyn Error>> {
    let decoder = GzipDecoder::new(input);
    let mut reader = quick_xml::Reader::from_reader(
        BufReader::with_capacity(BUFFER_SIZE, decoder)
    );

    let mut buf: Vec<u8> = Vec::new();
    let mut junk_buf: Vec<u8> = Vec::new();
    let mut tx = pool.begin().await?;
    let mut count = 0;

    loop {
        match reader.read_event_into_async(&mut buf).await {
            Err(e) => panic!(
                "Error at position {}: {:?}",
                reader.buffer_position(),
                e
            ),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"NATION" => {
                        let tag_bytes = read_to_end_into_buffer(
                            &mut reader, &e, &mut junk_buf
                        ).await?;

                        let str = String::from_utf8(tag_bytes)?;
                        let mut deserializer = Deserializer::from_str(&str);

                        match Nation::deserialize(&mut deserializer).map(Nation::finalize) {
                            Ok(nation) => {
                                nation.insert(&mut tx).await.unwrap_or_else(|err| {
                                    warn!("Error inserting nation {} into DB: {}", nation.canon_name, err);
                                });

                                count += 1;

                                if count >= NATION_BATCH_SIZE {
                                    tx.commit().await?;
                                    tx = pool.begin().await?;
                                    count = 0;
                                }
                            },
                            Err(err) => warn!("Nation deserialization error: {err}"),
                        }
                    },
                    b"REGION" => {
                        let tag_bytes = read_to_end_into_buffer(
                            &mut reader, &e, &mut junk_buf
                        ).await?;

                        let str = String::from_utf8(tag_bytes)?;
                        let mut deserializer = Deserializer::from_str(&str);

                        match Region::deserialize(&mut deserializer).map(Region::finalize) {
                            Ok(region) => {
                                region.insert(&mut tx).await.unwrap_or_else(|err| {
                                    warn!("Error inserting nation {} into DB: {}", region.canon_name, err);
                                });

                                count += 1;

                                if count >= REGION_BATCH_SIZE {
                                    tx.commit().await?;
                                    tx = pool.begin().await?;
                                    count = 0;
                                }
                            },
                            Err(err) => warn!("Region deserialization error: {err}"),
                        }
                    }
                _ => (),
                }
            }
            // Other Events are not important for us
            _ => (),
        }

        buf.clear();
    }

    if count > 0 {
        tx.commit().await?;
    }

    Ok(())
}

// sourced with small tweaks from https://capnfabs.net/posts/parsing-huge-xml-quickxml-rust-serde/
async fn read_to_end_into_buffer<R: AsyncBufRead + Unpin>(
    reader: &mut quick_xml::Reader<R>,
    start_tag: &BytesStart<'_>,
    junk_buf: &mut Vec<u8>,
) -> Result<Vec<u8>, quick_xml::Error> {
    let mut depth = 0;
    let mut output_buf: Vec<u8> = Vec::new();
    let mut w = quick_xml::Writer::new(&mut output_buf);
    let tag_name = start_tag.name();
    w.write_event(Event::Start(start_tag.clone()))?;

    loop {
        junk_buf.clear();
        let event = reader.read_event_into_async(junk_buf).await?;
        w.write_event(event.borrow())?;

        match event {
            Event::Start(e) if e.name() == tag_name => depth += 1,
            Event::End(e) if e.name() == tag_name => {
                if depth == 0 {
                    junk_buf.clear();
                    return Ok(output_buf);
                }
                depth -= 1;
            }
            Event::Eof => {
                junk_buf.clear();

                return Err(quick_xml::Error::IllFormed(
                    quick_xml::errors::IllFormedError::MissingEndTag(
                        String::from_utf8(tag_name.into_inner().to_vec()).unwrap()
                    )
                ));
            }
            _ => {}
        }
    }
}
