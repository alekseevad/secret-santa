use tide::Request;
use tide::Response;
use tide::prelude::*;
use std::env;
use std::string::String;
use mysql::PooledConn;
use mysql::Value::NULL;

#[derive(Debug, Deserialize)]
struct Participant {
    login: String,
    groupId: i64,
    is_admin: bool,
    santa_for: String,
}

#[async_std::main]
async fn main() -> tide::Result<()>
{
    let args: Vec<String> = env::args().collect();
    let address = &args[1];
    let listen = format!("{}", address);
    let mut server = tide::new();
    tide::log::start();

    server.at("/newMemb").post(new_member);

    server.listen(listen).await?;


    Ok(())
}

async fn new_member(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);

    let json_part { login, groupId, is_admin, santa_for, } = req.body_json().await?;
    addUser(&mut connect, &mut createParticipant(login, groupId.parse::<i64>().unwrap(), false, String::from("")).await).await;

    Ok((format!("Member added").into()))
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

async fn findUser(connect: &mut PooledConn, login: &String) -> bool {
    let resultQuery = connect.query(format!("SELECT* FROM santas_users WHERE login = \"{}\"", login)).unwrap();

    for resultList in resultQuery {
        println!("{}", "User is found");
        return true; // Если зашел в цикл, значит нашел что надо
    }

    println!("{}", "User is not found");
    return false;
}


async fn setGroupIdToUser(connect: &mut PooledConn, currentLogin: &String, groupId: i64, token: &String, res: &mut Response) -> bool {
    if currentLogin.is_empty() {
        res.set_body(format!("You cant change your group, login is empty"));
        return false;
    }

    let current_group = getGroupOfUser(connect, &currentLogin).await;

    if current_group == -1
    {
        res.set_body(format!("You are in game"));
        return false;
    }
    else if token == "join" && findGroup(connect, groupId).await {
        connect.query(format!("UPDATE santas_users SET groupId = {}, santa_for = null WHERE login = \"{}\"", groupId, currentLogin)).unwrap();
        res.set_body(format!("Added to group {}", groupId));
        return true;
    }
    else if token == "new_group" && !findGroup(connect, groupId).await {
        connect.query(format!("UPDATE santas_users SET groupId = {}, santa_for = null WHERE login = \"{}\"", groupId, currentLogin)).unwrap();
        res.set_body(format!("Added to group {}", groupId));
        return true;
    }
    else {
        res.set_body(format!("Group already exist"));
        return false;
    }
}

async fn createParticipant(login: String, groupId: i64, is_admin: bool, santa_for: String) -> Participant {
    return Participant {
        login,
        groupId, // DEFAULT: -1
        is_admin, // DEFAULT: FALSE
        santa_for,// DEFAULT: ""
    };
}

