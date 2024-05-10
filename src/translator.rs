use anyhow::anyhow;

pub trait Translator {
    async fn translate(&self, from_lang: &str, to_lang: &str, text: &str)
        -> anyhow::Result<String>;
}

pub struct GoogleTranslator {
    client: reqwest::Client,
}

impl Default for GoogleTranslator {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl GoogleTranslator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Translator for GoogleTranslator {
    async fn translate(
        &self,
        from_lang: &str,
        to_lang: &str,
        text: &str,
    ) -> anyhow::Result<String> {
        let res = self
            .client
            .get(&format!(
                "https://translate.google.com/m?sl={}&tl={}&q={}",
                from_lang, to_lang, text
            ))
            .send()
            .await?;
        let page = res.text().await?;

        let re = regex::Regex::new("(?s)<div class=\"result-container\">(.+?)</div>")?;
        match re.captures(&page) {
            None => Err(anyhow!("No translation found from page")),
            Some(capture) => match capture.get(1) {
                None => Err(anyhow!("No translation found from page")),
                Some(match_) => {
                    let translation = match_.as_str();
                    let translation = translation.replace("<span class=\"hps\">", "");
                    let translation = translation.replace("</span>", "");
                    let translation = translation.replace("<br>", "\n");

                    Ok(translation)
                }
            },
        }
    }
}
