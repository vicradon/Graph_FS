use super::util::*;
use crate::auth::check_access::{check_write_access, Authorized};
use crate::schema::GraphqlWebData;
use crate::{schema, utils};
use actix_multipart::Multipart;
use actix_web::http::header::HeaderValue;
use actix_web::HttpResponse;
use actix_web::{web, HttpRequest, Responder};
use std::io::{Error, Read};
use std::path::Path;

pub async fn read_file(
    req: HttpRequest,
    // Read file path
    info: web::Query<schema::PathQuery>,
    _: Authorized,
) -> Result<impl Responder, Error> {
    let file_path = std::path::Path::new(&info.path);
    utils::check_auth_path(file_path)?;
    let file = actix_files::NamedFile::open_async(file_path).await?;
    Ok(file.into_response(&req))
}

pub async fn upload(
    payload: Multipart,
    // Directory you want to add file
    info: web::Query<schema::PathQuery>,
    _: Authorized,
    req: HttpRequest,
    st: web::Data<GraphqlWebData>,
) -> Result<HttpResponse, Error> {
    if st.args.use_auth.is_some() && st.args.use_auth.unwrap() {
        let token = req
            .headers()
            .get("authorization")
            .unwrap_or(&HeaderValue::from(0))
            .to_str()
            .unwrap_or("")
            .to_string();
        if !check_write_access(st.args.clone(), &token) {
            return Ok(HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("Unauthorized to perform write operation"));
        }
    }

    let file_path = std::path::Path::new(&info.path);
    utils::check_auth_path(file_path)?;
    let upload_status = save_local_file(payload, file_path).await;
    let val = match upload_status {
        Ok(Some(val)) => {
            if val {
                Ok(HttpResponse::Ok()
                    .content_type("text/plain")
                    .body("update_succeeded"))
            } else {
                Ok(HttpResponse::BadRequest()
                    .content_type("text/plain")
                    .body("update_failed"))
            }
        }
        _ => Ok(HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("update_failed")),
    };
    val
}

pub async fn read_remote_file(
    sess: web::Data<GraphqlWebData>,
    info: web::Query<schema::PathQuery>,
    _: Authorized,
) -> Result<HttpResponse, Error> {
    let (mut remote_file, _) = sess
        .sess
        .as_ref()
        .unwrap()
        .scp_recv(Path::new(&info.path))?;

    let mut contents = Vec::new();
    remote_file.read_to_end(&mut contents)?;
    Ok(buffer_response(contents))
}

pub async fn upload_remote_file(
    sess: web::Data<GraphqlWebData>,
    payload: Multipart,
    // Directory you want to add file
    info: web::Query<schema::PathQuery>,
    _: Authorized,
    req: HttpRequest,
    st: web::Data<GraphqlWebData>,
) -> Result<HttpResponse, Error> {
    if st.args.use_auth.is_some() && st.args.use_auth.unwrap() {
        let token = req
            .headers()
            .get("authorization")
            .unwrap_or(&HeaderValue::from(0))
            .to_str()
            .unwrap_or("")
            .to_string();
        if !check_write_access(st.args.clone(), &token) {
            return Ok(HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("Unauthorized to perform write operation"));
        }
    }

    let file_path = std::path::Path::new(&info.path);
    utils::check_auth_path(file_path)?;
    let upload_status = save_remote_file(payload, sess.sess.as_ref().unwrap(), file_path).await;
    let val = match upload_status {
        Ok(Some(val)) => {
            if val {
                Ok(HttpResponse::Ok()
                    .content_type("text/plain")
                    .body("update_succeeded"))
            } else {
                Ok(HttpResponse::BadRequest()
                    .content_type("text/plain")
                    .body("update_failed"))
            }
        }
        _ => Ok(HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("update_failed")),
    };

    val
}
