use tide::Request;
use tide::Response;
use tide::prelude::*;
use std::env;
use std::string::String;
use mysql::PooledConn;
use mysql::Value::NULL;

#[async_std::main]
async fn main() -> tide::Result<()>
{
    let args: Vec<String> = env::args().collect();
    let address = &args[1];
    let listen = format!("{}", address);
    let mut server = tide::new();
    tide::log::start();
 
    server.listen(listen).await?;
 
    Ok(())
}

async fn addUser(connect: &mut PooledConn, participant: &mut Participant) -> bool { 
    if participant.login.is_empty() {
        return false;
    }
 
    if findUser(connect, &participant.login).await == true {
        return false;
    }
 
    let mut firstPartStrForSQL = String::from("INSERT INTO santas_users (login");
 
    let mut secondPartStrForSQL = String::from("VALUES (");
    secondPartStrForSQL.push('"');
    secondPartStrForSQL.push_str(participant.login.as_str());
    secondPartStrForSQL.push('"');
 
    if participant.groupId != -1 {
        firstPartStrForSQL.push_str(",groupId");
 
        secondPartStrForSQL.push(',');
        secondPartStrForSQL.push_str(participant.groupId.to_string().as_str());
    }
 
    firstPartStrForSQL.push_str(",is_admin");
 
    secondPartStrForSQL.push(',');
    secondPartStrForSQL.push_str(participant.is_admin.to_string().as_str());
 
    if !participant.santa_for.is_empty() {
        firstPartStrForSQL.push_str(",santa_for");

        secondPartStrForSQL.push_str(", \"");
        secondPartStrForSQL.push_str(participant.santa_for.to_string().as_str());
        secondPartStrForSQL.push('\"');
    }
 
 
    firstPartStrForSQL.push_str(") ");
    secondPartStrForSQL.push(')');
 
    let mut finalStrForSQL = String::from(firstPartStrForSQL);
    finalStrForSQL.push_str(secondPartStrForSQL.as_str());
 
    connect.query(finalStrForSQL).unwrap();
    return true;
}

fn connectToDataBase(urlBaseDate: &String) -> PooledConn {
    return mysql::Pool::new(urlBaseDate)
        .unwrap()
        .get_conn()
        .unwrap();
}

async fn createURLForConnectToDataBase() -> String { // URL типа: "mysql://root:password@localhost:3307/db_name"
    let args: Vec<String> = env::args().collect();
    return format!("{}", &args[2]);
}




