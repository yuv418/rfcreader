use std::borrow::BorrowMut;

use tl::{self, VDom};

#[derive(Default, Debug)]
pub struct RFCData {
    pub num: u32,
    pub title: String,
    pub text: Vec<String>,
}

impl RFCData {
    pub async fn new(num: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let resp = reqwest::get(format!("https://www.rfc-editor.org/rfc/rfc{}.html", num)).await?;
        if resp.status() == 200 {
            let rfc_html = resp.text().await?;
            let mut output = tl::parse(&rfc_html, tl::ParserOptions::default())?;
            let mut parsed_data = Self::parse(num, output)?;
            parsed_data.clean();
            Ok(parsed_data)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Non-200 HTTP status code",
            )))
        }
    }

    fn clean(&mut self) {
        // Could I do it in O(1) space complexity? Yes. Am I lazy? Yes.
        let mut text_vec = vec![];

        let mut text_buffer = String::new();
        let mut last_two_chars = ['\0', '\0'];
        let mut l2c_i = 0;

        for line in &self.text {
            // A very bad way to check if something is an HTML tag
            if !line.starts_with("<") {
                for c in line.chars() {
                    last_two_chars[l2c_i] = c;

                    l2c_i += 1;
                    l2c_i %= 2;

                    if last_two_chars[0] != '\n' || last_two_chars[1] != '\n' {
                        text_buffer.push(c);
                    } else {
                        text_vec.push(format!("<p>{}</p>", text_buffer));
                        text_buffer.clear();
                    }
                }
            } else {
                text_buffer += line;
            }
        }
        self.text = text_vec;
    }

    fn parse(num: u32, mut vdom: VDom) -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = vdom.parser_mut();

        let mut title = "Title Missing".to_owned();
        let mut text = vec![];

        if let Some(handle) = vdom.query_selector("pre") {
            for pre in handle {
                // There should only be one but
                for el in pre.get(parser) {
                    if let Some(children_set) = el.children() {
                        for child in children_set.top().to_vec() {
                            if let Some(child) = child.get(parser) {
                                match child {
                                    tl::Node::Tag(tag) => {
                                        if tag.attributes().is_class_member("h1") {
                                            title = tag.inner_text(parser).to_string();
                                            continue;
                                        } else if tag.attributes().is_class_member("h4") {
                                            let mut name = tag.name_mut().as_utf8_str();
                                            name = std::borrow::Cow::Borrowed("h4");
                                        } else {
                                            text.push(tag.raw().as_utf8_str().to_string());
                                        }
                                    }
                                    tl::Node::Raw(bytes) => {
                                        text.push(bytes.as_utf8_str().to_string());
                                    }
                                    // tl::Node::Comment(comment) => todo!(),
                                    _ => {}
                                }
                                //println!("{:?}", child);
                            }
                        }
                    }
                }
            }

            // println!("Title: {}", title);
            // println!("Text: {:?}", text);
        }

        Ok(RFCData { num, title, text })

        /*let title = if let Some(el) = vdom.query_selector(".h1")?.collect().get(0) {
            el.inner_text(parser)
        } else {
            return Err();
        };*/
    }
}
