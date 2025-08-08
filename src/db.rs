use {
    chrono::{DateTime, Utc},
    reqwest::{Client, Method, StatusCode, Url, header::CONTENT_TYPE},
};
pub struct Metrics {
    slot: u64,
    tvc_credits: u64,
    avg_latency: u64,
    efficency: f64,
    missed_credits: u64,
    low_latency_rate: f64,
    tx: String,
    vote_rate: f64,
}

impl Metrics {
    fn escape_tag_value(value: &str) -> String {
        value
            .replace('\\', r"\\")
            .replace(' ', r"\ ")
            .replace(',', r"\,")
            .replace('=', r"\=")
    }
    fn to_line_protocol(&self) -> String {
        // format!(
        //     "home,room={} temp={},hum={},co={}i {}",
        //     Self::escape_tag_value(&self.room),
        //     self.temp,
        //     self.hum,
        //     self.co,
        //     self.timestamp.timestamp()
        // )
        "".to_string()
    }
}
pub async fn write_data(lines: Vec<Metrics>) -> Result<(), anyhow::Error> {
    // POST /api/v3/write_lp?db=mydb&precision=nanosecond&accept_partial=true&no_sync=false
    let base_url = "https://localhost:3000";
    let db_name = "db";

    let db_url = format!(
        "{:?}/api/v3/write_lp?db={:?}&precision=ms&accept_partial=true",
        base_url, db_name,
    );

    let client = Client::new();

    let lines_db: Vec<String> = lines.iter().map(|r| r.to_line_protocol()).collect();

    let body = lines_db.join("\n");

    let _response = match client
        .post(db_url)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error: unable to commit meterics");

            return Err(anyhow::Error::msg("Error: unable to commit meterics"));
        }
    };

    Ok(())
}
