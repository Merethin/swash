use serde::{Serialize, Deserialize, Deserializer};
use sqlx::PgTransaction;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Nation {
    name: String,
    #[serde(skip_deserializing)]
    pub canon_name: String,
    #[serde(rename = "TYPE")]
    classification: String,
    fullname: String,
    motto: String,
    category: String,
    unstatus: String,
    #[serde(skip_deserializing)]
    is_wa: bool,
    #[serde(skip_deserializing)]
    is_delegate: bool,
    #[serde(rename = "ENDORSEMENTS")]
    #[serde(deserialize_with = "deserialize_comma_list")]
    endorsements: Vec<String>,
    issues_answered: i64,
    #[serde(rename = "FREEDOM")]
    freedoms: Freedoms,
    #[serde(rename = "REGION")]
    #[serde(deserialize_with = "deserialize_canon")]
    region: String,
    population: i64,
    tax: f64,
    animal: String,
    currency: String,
    demonym: String,
    demonym2: String,
    demonym2plural: String,
    #[serde(rename = "FLAG")]
    flag_url: String,
    #[serde(rename = "MAJORINDUSTRY")]
    major_industry: String,
    #[serde(rename = "GOVTPRIORITY")]
    govt_priority: String,
    #[serde(rename = "GOVT")]
    government: Government,
    founded: String,
    firstlogin: i64,
    lastlogin: i64,
    lastactivity: String,
    influence: String,
    influencenum: f64,
    #[serde(rename = "FREEDOMSCORES")]
    freedom_scores: FreedomScores,
    publicsector: f64,
    #[serde(rename = "DEATHS")]
    #[serde(deserialize_with = "deserialize_deaths")]
    deaths: Vec<Cause>,
    leader: String,
    capital: String,
    religion: String,
    factbooks: i64,
    dispatches: i64,
    dbid: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Freedoms {
    #[serde(rename(deserialize = "CIVILRIGHTS"))]
    civilrights: String,
    #[serde(rename(deserialize = "ECONOMY"))]
    economy: String,
    #[serde(rename(deserialize = "POLITICALFREEDOM"))]
    polfreedom: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FreedomScores {
    #[serde(rename(deserialize = "CIVILRIGHTS"))]
    civilrights: i64,
    #[serde(rename(deserialize = "ECONOMY"))]
    economy: i64,
    #[serde(rename(deserialize = "POLITICALFREEDOM"))]
    polfreedom: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Government {
    #[serde(rename(deserialize = "ADMINISTRATION"))]
    admin: f64,
    #[serde(rename(deserialize = "DEFENCE"))]
    defence: f64,
    #[serde(rename(deserialize = "EDUCATION"))]
    education: f64,
    #[serde(rename(deserialize = "ENVIRONMENT"))]
    environment: f64,
    #[serde(rename(deserialize = "HEALTHCARE"))]
    healthcare: f64,
    #[serde(rename(deserialize = "COMMERCE"))]
    commerce: f64,
    #[serde(rename(deserialize = "INTERNATIONALAID"))]
    aid: f64,
    #[serde(rename(deserialize = "LAWANDORDER"))]
    law: f64,
    #[serde(rename(deserialize = "PUBLICTRANSPORT"))]
    transport: f64,
    #[serde(rename(deserialize = "SOCIALEQUALITY"))]
    equality: f64,
    #[serde(rename(deserialize = "SPIRITUALITY"))]
    spirituality: f64,
    #[serde(rename(deserialize = "WELFARE"))]
    welfare: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cause {
    #[serde(rename(deserialize = "@type"))]
    kind: String,
    #[serde(rename(deserialize = "$text"))]
    percentage: f64,
}

impl Nation {
    pub fn finalize(mut self) -> Self {
        self.canon_name = self.name.to_lowercase().replace(' ', "_");
        self.is_delegate = self.unstatus == "WA Delegate";
        self.is_wa = self.is_delegate || self.unstatus == "WA Member";
        self
    }

    pub async fn insert(
        &self, tx: &mut PgTransaction<'_>
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query(r"INSERT INTO nations_dump (
                dbid, name, canon_name, classification, fullname, motto, category, unstatus,
                is_wa, is_delegate, endorsements, issues_answered, freedoms, region, population,
                tax, animal, currency, demonym, demonym2, demonym2plural, flag_url,
                major_industry, govt_priority, government, founded, firstlogin, lastlogin, lastactivity,
                influence, influencenum, freedom_scores, public_sector, deaths, leader, capital,
                religion, factbooks, dispatches) VALUES 
                ($1,$2,$3,$4,$5,$6,$7,$8,
                $9,$10,$11,$12,$13,$14,$15,
                $16,$17,$18,$19,$20,$21,$22,
                $23,$24,$25,$26,$27,$28,$29,
                $30,$31,$32,$33,$34,$35,$36,
                $37,$38,$39) ON CONFLICT (canon_name) DO UPDATE SET
                dbid = EXCLUDED.dbid,
                name = EXCLUDED.name,
                canon_name = EXCLUDED.canon_name,
                classification = EXCLUDED.classification,
                fullname = EXCLUDED.fullname,
                motto = EXCLUDED.motto,
                category = EXCLUDED.category,
                unstatus = EXCLUDED.unstatus,
                is_wa = EXCLUDED.is_wa,
                is_delegate = EXCLUDED.is_delegate,
                endorsements = EXCLUDED.endorsements,
                issues_answered = EXCLUDED.issues_answered,
                freedoms = EXCLUDED.freedoms,
                region = EXCLUDED.region,
                population = EXCLUDED.population,
                tax = EXCLUDED.tax,
                animal = EXCLUDED.animal,
                currency = EXCLUDED.currency,
                demonym = EXCLUDED.demonym,
                demonym2 = EXCLUDED.demonym2,
                demonym2plural = EXCLUDED.demonym2plural,
                flag_url = EXCLUDED.flag_url,
                major_industry = EXCLUDED.major_industry,
                govt_priority = EXCLUDED.govt_priority,
                government = EXCLUDED.government,
                founded = EXCLUDED.founded,
                firstlogin = EXCLUDED.firstlogin,
                lastlogin = EXCLUDED.lastlogin,
                lastactivity = EXCLUDED.lastactivity,
                influence = EXCLUDED.influence,
                influencenum = EXCLUDED.influencenum,
                freedom_scores = EXCLUDED.freedom_scores,
                public_sector = EXCLUDED.public_sector,
                deaths = EXCLUDED.deaths,
                leader = EXCLUDED.leader,
                capital = EXCLUDED.capital,
                religion = EXCLUDED.religion,
                factbooks = EXCLUDED.factbooks,
                dispatches = EXCLUDED.dispatches")
        .bind(self.dbid)
        .bind(&self.name)
        .bind(&self.canon_name)
        .bind(&self.classification)
        .bind(&self.fullname)
        .bind(&self.motto)
        .bind(&self.category)
        .bind(&self.unstatus)
        .bind(self.is_wa)
        .bind(self.is_delegate)
        .bind(&self.endorsements)
        .bind(self.issues_answered)
        .bind(serde_json::to_value(&self.freedoms)?)
        .bind(&self.region)
        .bind(self.population)
        .bind(self.tax)
        .bind(&self.animal)
        .bind(&self.currency)
        .bind(&self.demonym)
        .bind(&self.demonym2)
        .bind(&self.demonym2plural)
        .bind(&self.flag_url)
        .bind(&self.major_industry)
        .bind(&self.govt_priority)
        .bind(serde_json::to_value(&self.government)?)
        .bind(&self.founded)
        .bind(self.firstlogin)
        .bind(self.lastlogin)
        .bind(&self.lastactivity)
        .bind(&self.influence)
        .bind(self.influencenum)
        .bind(serde_json::to_value(&self.freedom_scores)?)
        .bind(self.publicsector)
        .bind(serde_json::to_value(&self.deaths)?)
        .bind(&self.leader)
        .bind(&self.capital)
        .bind(&self.religion)
        .bind(self.factbooks)
        .bind(self.dispatches)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

fn deserialize_comma_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();

    if s.trim().is_empty() {
        return Ok(Vec::new());
    }

    Ok(s.split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn deserialize_canon<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.to_lowercase().replace(" ", "_"))
}

#[derive(Deserialize)]
struct DeathsWrapper {
    #[serde(rename = "CAUSE")]
    causes: Vec<Cause>,
}

fn deserialize_deaths<'de, D>(deserializer: D) -> Result<Vec<Cause>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wrapper = DeathsWrapper::deserialize(deserializer)?;
    Ok(wrapper.causes)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Region {
    name: String,
    #[serde(skip_deserializing)]
    pub canon_name: String,
    #[serde(rename = "FACTBOOK")]
    #[serde(deserialize_with = "deserialize_html_decoded")]
    factbook: String,
    numnations: i64,
    #[serde(rename = "NATIONS")]
    #[serde(deserialize_with = "deserialize_colon_list")]
    nations: Vec<String>,
    delegate: String,
    delegatevotes: i64,
    delegateauth: String,
    frontier: i64,
    founder: String,
    governor: String,
    #[serde(rename = "OFFICERS")]
    #[serde(deserialize_with = "deserialize_officers")]
    officers: Vec<Officer>,
    power: String,
    magnetism: f64,
    #[serde(rename = "FLAG")]
    flag_url: String,
    #[serde(rename = "BANNER")]
    banner_id: String,
    #[serde(rename = "BANNERURL")]
    banner_url: String,
    #[serde(rename = "EMBASSIES")]
    #[serde(deserialize_with = "deserialize_embassies")]
    embassies: Vec<String>,
    lastupdate: i64,
    lastmajorupdate: i64,
    lastminorupdate: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
struct Officer {
    nation: String,
    office: String,
    authority: String,
    time: i64,
    by: String,
    order: i64,
}

impl Region {
    pub fn finalize(mut self) -> Self {
        self.canon_name = self.name.to_lowercase().replace(' ', "_");
        self
    }

    pub async fn insert(&self, tx: &mut PgTransaction<'_>) -> Result<(), Box<dyn Error>> {
        sqlx::query(r"INSERT INTO regions_dump (
                name, canon_name, factbook, numnations, nations, delegate, delegatevotes,
                delegateauth, frontier, founder, governor, officers, power, magnetism,
                flag_url, banner_id, banner_url, embassies, lastupdate, lastmajorupdate,
                lastminorupdate) VALUES ($1,$2,$3,$4,$5,$6,$7,
                $8,$9,$10,$11,$12,$13,$14,
                $15,$16,$17,$18,$19,$20,$21)
                ON CONFLICT (canon_name) DO UPDATE SET
                name = EXCLUDED.name,
                factbook = EXCLUDED.factbook,
                numnations = EXCLUDED.numnations,
                nations = EXCLUDED.nations,
                delegate = EXCLUDED.delegate,
                delegatevotes = EXCLUDED.delegatevotes,
                delegateauth = EXCLUDED.delegateauth,
                frontier = EXCLUDED.frontier,
                founder = EXCLUDED.founder,
                governor = EXCLUDED.governor,
                officers = EXCLUDED.officers,
                power = EXCLUDED.power,
                magnetism = EXCLUDED.magnetism,
                flag_url = EXCLUDED.flag_url,
                banner_id = EXCLUDED.banner_id,
                banner_url = EXCLUDED.banner_url,
                embassies = EXCLUDED.embassies,
                lastupdate = EXCLUDED.lastupdate,
                lastmajorupdate = EXCLUDED.lastmajorupdate,
                lastminorupdate = EXCLUDED.lastminorupdate")
        .bind(&self.name)
        .bind(&self.canon_name)
        .bind(&self.factbook)
        .bind(self.numnations)
        .bind(&self.nations)
        .bind(&self.delegate)
        .bind(self.delegatevotes)
        .bind(&self.delegateauth)
        .bind(self.frontier)
        .bind(&self.founder)
        .bind(&self.governor)
        .bind(serde_json::to_value(&self.officers)?)
        .bind(&self.power)
        .bind(self.magnetism)
        .bind(&self.flag_url)
        .bind(&self.banner_id)
        .bind(&self.banner_url)
        .bind(&self.embassies)
        .bind(self.lastupdate)
        .bind(self.lastmajorupdate)
        .bind(self.lastminorupdate)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

fn deserialize_colon_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();

    if s.trim().is_empty() {
        return Ok(Vec::new());
    }

    Ok(s.split(':')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(ToString::to_string)
        .collect())
}

#[derive(Deserialize)]
struct OfficersWrapper {
    #[serde(rename = "OFFICER")]
    #[serde(default)]
    officers: Vec<Officer>,
}

fn deserialize_officers<'de, D>(deserializer: D) -> Result<Vec<Officer>, D::Error>
where
    D: Deserializer<'de>,
{
    let wrapper = OfficersWrapper::deserialize(deserializer)?;
    Ok(wrapper.officers)
}

#[derive(Deserialize)]
struct EmbassyWrapper {
    #[serde(rename = "EMBASSY")]
    #[serde(default)]
    embassies: Vec<String>,
}

fn deserialize_embassies<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let wrapper = EmbassyWrapper::deserialize(deserializer)?;
    Ok(wrapper.embassies.into_iter().map(|v| v.to_lowercase().replace(' ', "_")).collect())
}

fn deserialize_html_decoded<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(html_escape::decode_html_entities(&s).to_string())
}