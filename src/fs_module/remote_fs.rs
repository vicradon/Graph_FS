use std::path::Path;

use juniper::FieldResult;

use crate::{
    schema::{Context, Message},
    utils::check_auth_path,
};
pub struct RemoteFsQuery;

#[juniper::graphql_object(context = Context)]
impl RemoteFsQuery {
    #[graphql(description = "create file")]
    fn create_file(context: &Context, path: String) -> FieldResult<Message> {
        let path = Path::new(&path);
        check_auth_path(&path)?;
        let sftp = context.sess.sftp()?;
        sftp.create(path)?;
        let return_msg = format!("{} created successfully", path.to_str().unwrap());
        Ok(Message::new(String::from(return_msg)))
    }

    #[graphql(
        description = "create directory. Set mode optionally, would default to allow user read and write without sudo"
    )]
    fn create_dir(context: &Context, path: String, mode: Option<i32>) -> FieldResult<Message> {
        let path = Path::new(&path);
        check_auth_path(&path)?;
        let sftp = context.sess.sftp()?;
        //use 777 as mode if none provided
        sftp.mkdir(path, mode.unwrap_or(1000))?;
        let return_msg = format!("{} created successfully", path.to_str().unwrap());
        Ok(Message::new(String::from(return_msg)))
    }
}
