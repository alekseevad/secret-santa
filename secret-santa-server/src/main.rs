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

#[derive(Debug, Deserialize)]
struct json_group {
    login: String,
    groupId: String,
}

#[derive(Debug, Deserialize)]
struct json_login {
    login: String,
}

#[derive(Debug, Deserialize)]
struct json_admin {
    admin: String,
    login: String,
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

    server.at("/newGroup").post(new_group);
    
    server.at("/joinGroup").post(join_group);
    
    server.at("/leftGroup").post(left_group);
    
    server.at("/setAdmin").post(set_admin);
    
    server.at("/deleteGroup").post(delete_group);
    
    server.at("/check_santa").post(check_santa);
    
    server.at("/secretGameSanta").post(startGameSecretSanta);

    server.listen(listen).await?;


    Ok(())
}

async fn new_member(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);

    let json_part { login, groupId, is_admin, santa_for, } = req.body_json().await?;
    addUser(&mut connect, &mut createParticipant(login, groupId.parse::<i64>().unwrap(), false, String::from("")).await).await;

    Ok((format!("Member added").into()))
}

async fn new_group(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);

    let json_group { login, groupId, } = req.body_json().await?;

    let group_id = groupId.parse::<i64>().unwrap();
    println!("{} {}", login, group_id);
    let token = "new_group".to_string();
    let mut res = Response::new(200);
    if setGroupIdToUser(&mut connect, &login, group_id, &token, &mut res).await == true {
        let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
        setUserToAdminInGroup(&mut connect, &login, true).await;
    }

    Ok(res)
}

async fn join_group(mut req: Request<()>) -> tide::Result {
    let args: Vec<String> = env::args().collect();
    let url = format!("{}", &args[2]);
    let mut connect = connectToDataBase(&url);
 
    let json_group { login, groupId, } = req.body_json().await?;
    let group_id = groupId.parse::<i64>().unwrap();
    println!("{} {}", login, group_id);
    let token = "join".to_string();
    let mut res = Response::new(200);
    setGroupIdToUser(&mut connect, &login, group_id, &token, &mut res).await;

    Ok(res)
}

async fn left_group(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
 
    let json_login { login } = req.body_json().await?;
    let mut resp: String = String::new();
    if isAdmin(&mut connect, &login).await == false {
        if getGroupOfUser(&mut connect, &login).await != -1 {
            setUserGroupToNull(&mut connect, &login).await;
            resp = "You left your group".to_string();
        } else {
            resp = "Error: you are not in group. Join the group or create a new one and try again.".to_string();
        }
    } else {
        resp = "Error: you are an admin. At first, resign from your duties.".to_string();
    }
 
    Ok((format!("{}", resp).into()))
}

async fn set_admin(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
    let mut resp = String::new();
    let json_admin { admin, login } = req.body_json().await?;
 
    if isAdmin(&mut connect, &admin).await == true {
        let adm: i64 = getGroupOfUser(&mut connect, &admin.trim_end().to_string()).await;
        let log: i64 = getGroupOfUser(&mut connect, &login.trim_end().to_string()).await;
 
        if adm == log {
            connect.query(format!("UPDATE santas_users SET is_admin = {} WHERE login = \"{}\"", true, login.trim_end().to_string())).unwrap();
            resp = "Admin set".to_string();
        } else {
            resp = "Error: this user is not in your group.".to_string();
        }
    } else {
        resp = "Error: you are not an admin.".to_string();
    }
 
 
    Ok((format!("{}", resp).into()))
}

async fn delete_group(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
    let mut resp = String::new();
    let json_login {login} = req.body_json().await?;
    let group_id = getGroupOfUser(&mut connect, &login).await;

    if(isAdmin(&mut connect, &login).await==true) {
        setNullByGroup(&mut connect, group_id).await;
        resp = format!("{} deleted", group_id);
    }
    else{
        resp = "Error: you are not an admin.".to_string();
    }
    Ok((format!("{}", resp).into()))
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

async fn startGameSecretSanta(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
    let json_login { login } = req.body_json().await?;
 
    startGame(&mut connect, &login, &createURLForConnectToDataBase().await).await;
    Ok((format!("Game started").into()))
}

async fn startGame(connect: &mut PooledConn, currentLogin: &String, url: &String)-> bool {
    if currentLogin.is_empty() {
        return false;
    }
 
    if isAdmin(connect, currentLogin).await == false {
        return false;
    }
 
    let resultQueryList = connect.query(format!("SELECT login FROM santas_users WHERE groupId = (SELECT groupId FROM santas_users WHERE login = \"{}\")", currentLogin)).unwrap();
 
    let mut vec = Vec::new();
    let mut flag = false;
 
    for resultList in resultQueryList {
        let currentRow = resultList.unwrap().unwrap();
        flag = true;
 
        for valueOfRow in currentRow {
            let mut currentLogin = valueOfRow.as_sql(false);
 
            currentLogin.pop();
            if currentLogin.len() > 0 {
                currentLogin.remove(0);
            }
            vec.push(currentLogin);
        }
    }
 
    if flag == false {
        return false;
    }
 
    if vec.len() == 1 {
        return false;
    }
 
    let mut firstLogin = vec.get(0).unwrap();
    println!("{}", firstLogin);
	
    for currentLogin in vec.iter() {
        if cntIteration == 0 {
            cntIteration += 1;
            continue;
        }
 
        setSecretSantaToUser(connect, &prevLogin.trim_end().to_string(), &currentLogin.trim_end().to_string()).await;
        prevLogin = currentLogin;
    }
 
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
 
    let mut group_id = getNumberGroupUser(&mut connect, &currentLogin.trim_end().to_string()).await;
 
    setSecretSantaToUser(&mut connect, &prevLogin.trim_end().to_string(), &firstLogin.trim_end().to_string()).await;
 
    setNullAllFields(&mut connect, group_id).await;
    return true;
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

async fn setNullAllFields(connect: &mut PooledConn, groupId: i64) -> bool {
    if findGroup(connect, groupId).await == false {
        return false;
    }
 
    let handler = connect.query(format!("UPDATE santas_users SET
            groupID = -1,
            is_admin = null
            WHERE groupID = {}", groupId));
    return true;
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

async fn setUserToAdminInGroup(connect: &mut PooledConn, currentLogin: &String, isAdmin: bool) {
    if currentLogin.is_empty() {
        return;
    }

    let mut group_id = getNumberGroupUser(connect, currentLogin).await;
    let mut connect = &mut connectToDataBase(&createURLForConnectToDataBase().await);
    if isAdmin == false {
        connect.query(format!("UPDATE santas_users SET is_admin = {} WHERE login = \"{}\"", isAdmin, currentLogin)).unwrap();
    }

    if isAdmin == true {
        connect.query(format!("UPDATE santas_users SET is_admin = {} WHERE login = \"{}\"", isAdmin as u8, currentLogin)).unwrap();
    }
}

async fn getNumberGroupUser(connect: &mut PooledConn, loginUser: &String) -> i64 {
    let resultQuery = connect.query(format!("SELECT groupId FROM santas_users WHERE login = \"{}\"", loginUser)).unwrap();

    for resultList in resultQuery {
        let currentRow = resultList.unwrap().unwrap();
        for valueOfRow in currentRow {
            if valueOfRow == NULL {
                return -1;
            }

            let mut chars = valueOfRow.as_sql(false);

            chars.pop();
            if chars.len() > 0 {
                chars.remove(0);
            }

            return chars.parse().expect("Parse ERROR :(");
        }
    }

    return -1;
}

async fn findGroup(connect: &mut PooledConn, groupId: i64) -> bool {
    let resultQuery = connect.query(format!("SELECT* FROM santas_users WHERE groupId = {}", groupId)).unwrap();

    for resultList in resultQuery {
        println!("{}", "Group is found");
        return true; // Если зашел в цикл, значит нашел что надо
    }

    println!("{}", "Group is not found");
    return false;
}

async fn getGroupOfUser(connect: &mut PooledConn, loginUser: &String) -> i64 {
    let resultQuery = connect.query(format!("SELECT groupId FROM santas_users WHERE login = \"{}\"", loginUser)).unwrap();

    for resultList in resultQuery {
        let currentRow = resultList.unwrap().unwrap();

        for valueOfRow in currentRow {
            if valueOfRow == NULL {
                return 0;
            }

            let mut chars = valueOfRow.as_sql(false);

            chars.pop(); // remove last
            if chars.len() > 0 {
                chars.remove(0); // remove first
            }

            return chars.parse().expect("Parse ERROR :(");
        }
    }

    return -1;
}

async fn isAdmin(connect: &mut PooledConn, currentLogin: &String) -> bool {
    if currentLogin.is_empty() {
        return false;
    }
 
    let resultQueryList = connect.query(format!("SELECT is_admin FROM santas_users WHERE login = \"{}\"", currentLogin)).unwrap();
 
    for resultList in resultQueryList {
        let currentRow = resultList.unwrap().unwrap();
 
        for valueOfRow in currentRow {
            if valueOfRow == NULL {
                return false;
            }
            println!("{}", valueOfRow.as_sql(false).trim_end());
 
            if valueOfRow.as_sql(true).trim() == "'1'" {
                return true;
            }
 
            return false;
        }
    }
 
    return false;
}
 
async fn setUserGroupToNull(connect: &mut PooledConn, currentLogin: &String) {
    if currentLogin.is_empty() {
        return;
    }
 
    connect.query(format!("UPDATE santas_users SET groupId = null WHERE login = \"{}\"", currentLogin)).unwrap();
}

async fn setSecretSantaToUser(connect: &mut PooledConn, currentLogin: &String, nameSecretSanta: &String) {
    if currentLogin.is_empty() {
        return;
    }
 
    connect.query(format!("UPDATE santas_users SET santa_for = \"{}\" WHERE login = \"{}\"", nameSecretSanta, currentLogin)).unwrap();
}

async fn check_santa(mut req: Request<()>) -> tide::Result {
    let mut connect = connectToDataBase(&createURLForConnectToDataBase().await);
 
    let mut resp: String = String::new();
 
    let json_login { login } = req.body_json().await?;
    let resultQuery = connect.query(format!("SELECT santa_for FROM santas_users WHERE login = \"{}\"", login)).unwrap();
 
    for resultList in resultQuery {
        let currentRow = resultList.unwrap().unwrap();
        for valueOfRow in currentRow {
            if valueOfRow != NULL {
                resp = valueOfRow.as_sql(false);
            }
        }
    }
 
    Ok(format!("{} is your santa", resp).into())
}

async fn countAdmins(connect: &mut PooledConn, groupId: i64) -> bool {
    let cntAdmins = connect.query(format!("SELECT COUNT(is_admin) FROM santas_users WHERE (is_admin = 1 AND groupId = {})", groupId)).unwrap();
    let mut count_query: i8 = 0;
    for resultList in cntAdmins {
        let currentRow = resultList.unwrap().unwrap();
        for valueOfRow in currentRow {
            if valueOfRow != NULL {
                let mut count_str = valueOfRow.as_sql(false);
                count_str.pop();   // remove last
                if count_str.len() > 0 {
                    count_str.remove(0); // remove first
                }
                count_query = count_str.parse::<i8>().unwrap();
            }
            println!("{}", count_query);
        }
    }
    
    if count_query > 1 {
        return true;
    }
    
    return false;
}

fn showFullDateBase(connect: &mut PooledConn) {
    let resultQuery = connect.query("SELECT* FROM santas_users").unwrap();

    for resultList in resultQuery {
        let currentRow = resultList.unwrap().unwrap();
        
        for valueOfRow in currentRow { 
            if valueOfRow != NULL {
                print!("{}", valueOfRow.as_sql(false));
            }
        }
        println!();
    }
}
