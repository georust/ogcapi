use crate::common::link::ContentType;
use crate::common::Exception;
use crate::features::service::State;
use tide::{Body, Request, Response, Result};

pub async fn handle_root(req: Request<State>) -> Result {
    let url = req.url();

    let mut landing_page = req.state().root.clone();
    for link in landing_page.links.iter_mut() {
        link.href = format!("{}{}", url, link.href.trim_matches('/'));
    }

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&landing_page)?);
    Ok(res)
}

pub async fn handle_api(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_content_type(ContentType::OPENAPI);
    res.set_body(Body::from_json(&req.state().openapi)?);
    Ok(res)
}

pub async fn show_redoc(req: Request<State>) -> Result {
    let mut url = req.url().to_owned();

    url.set_query(None);
    let api_url = url.to_string().replace("redoc", "api");

    let mut res = Response::new(200);
    res.set_content_type(tide::http::mime::HTML);
    res.set_body(format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>ReDoc</title>
            <!-- needed for adaptive design -->
            <meta charset="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">

            <!--
            ReDoc doesn't change outer page styles
            -->
            <style>
            body {{
                margin: 0;
                padding: 0;
            }}
            </style>
        </head>
        <body>
            <redoc spec-url="{}"></redoc>
            <script src="https://cdn.jsdelivr.net/npm/redoc@next/bundles/redoc.standalone.js"> </script>
        </body>
        </html>
        "#,
        api_url
    ));
    Ok(res)
}

pub async fn handle_conformance(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&req.state().conformance)?);
    Ok(res)
}

pub async fn exception(result: Result) -> Result {
    match result {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res)
            } else {
                println!("WTF:\n{:#?}", res);
                panic!()
            }
        }
        Err(err) => {
            let status = err.status();
            let mut res = Response::new(status);
            let exception = Exception {
                code: status.to_string(),
                description: Some(err.to_string()),
            };
            res.set_body(Body::from_json(&exception)?);
            Ok(res)
        }
    }
}

pub async fn handle_favicon(_: Request<State>) -> Result {
    let mut res = Response::new(200);
    res.set_body(Body::from_file("favicon.ico").await?);
    Ok(res)
}
