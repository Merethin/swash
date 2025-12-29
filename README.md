# swash

NationStates data dump downloader and streaming parser

## Description

Swash downloads the daily data dumps from NationStates, parses them, and stores the output in a PostgreSQL database.

To avoid hogging memory, it does not read/download the entire file into memory, but streams the download and progressively stores parsed rows in the database, maintaining a rather low memory usage.

This program is meant as an auxiliary for other programs that want data from the daily data dumps but don't want to open and parse a large XML file all the time, nor download it themselves. Running database queries is a lot more flexible than searching for data in a large XML file, as an example:

`SELECT canon_name FROM nations_dump WHERE is_wa = TRUE AND region = 'the_east_pacific';`

will quickly return all WA nations in The East Pacific.

## Transformations

Some fields are slightly modified, parsed further, renamed, or added:

#### Nation dump transformations

`canon_name` is added, as an API-friendly version of the `name` field (`name`="Crazy girl" will yield `canon_name`="crazy_girl").

`<TYPE></TYPE>` is renamed to `classification`.

`is_wa` and `is_delegate` are added as boolean fields computed from the `unstatus` field (`unstatus`="WA Member" will yield `is_wa`=TRUE, `is_delegate`=FALSE).

The comma-separated `endorsements` list is parsed into an array of nation names (`endorsements`="testlandia,maxtopia" will yield `endorsements`=["testlandia", "maxtopia"]).

`<FREEDOM></FREEDOM>` is renamed to `freedoms`.

`region` is canonicalized/made API-friendly (`region`="The North Pacific" will yield `region`="the_north_pacific").

`<FLAG></FLAG>` is renamed to `flag_url`.

`<MAJORINDUSTRY></MAJORINDUSTRY>` is renamed to `major_industry`.

`<GOVTPRIORITY></GOVTPRIORITY>` is renamed to `govt_priority`.

`<GOVT></GOVT>` is renamed to `government`.

`<FREEDOMSCORES></FREEDOMSCORES>` is renamed to `freedom_scores`.

#### Region dump transformations

`canon_name` is added, as an API-friendly version of the `name` field (`name`="the Plains of Perdition" will yield `canon_name`="the_plains_of_perdition").

`factbook`'s HTML entities are decoded (`factbook`="Hello \&lt;world\&gt;" will yield `factbook`="Hello &lt;world&gt;").

The colon-separated `nations` list is parsed into an array of nation names (`nations`="testlandia:maxtopia" will yield `nations`=["testlandia", "maxtopia"]).

`<FLAG></FLAG>` is renamed to `flag_url`.

`<BANNER></BANNER>` is renamed to `banner_id`.

`<BANNERURL></BANNERURL>` is renamed to `banner_url`.

The values in `embassies` are canonicalized/made API-friendly (`embassies`=["Testregionia", "The Rejected Realms"] will yield `embassies`=["testregionia", "the_rejected_realms"]).

## Setting up

You will need to create the necessary database tables yourself prior to running the program.

The necessary PostgreSQL commands are found in [nations.sql](sql/nations.sql) and [regions.sql](sql/regions.sql).

Create an .env file with a `DATABASE_URL` (set to the URL necessary to connect to your PostgreSQL database). The `NS_USER_AGENT` variable, which must be set to your main nation name, can be set in the .env file or directly in the environment.

## Running

Make sure to clone the repository recursively (`git clone --recursive`) or download submodules after cloning! (`git submodule init && git submodule update --remote`)

Compile with `cargo build --release`.

Run with `./target/release/swash` (if the user agent is not set in .env, `NS_USER_AGENT=[YOUR MAIN NATION NAME] ./target/release/swash`). 

The recommended usage is to put this in a cron job to run daily around 1 hour after major update, and run all programs depending on it after that (using a shell script, for example).

## Other flags

Running with `-k/--keep` will not empty the data in both tables prior to running, instead replacing conflicting values (which will be most of them). This usage is not recommended as it will mix data from several data dumps.

Running with `-p/--path DIRECTORY` will parse the data from the `nations.xml.gz` and `regions.xml.gz` files in the specified directory instead of downloading them from NationStates. Useful for testing, to avoid downloading these files several times.