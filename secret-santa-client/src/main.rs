use std::collections::HashMap;
use std::env;
use reqwest::Client;
use tokio::io;
use std::string::String;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let address = &args[1];
    let port = &args[2];
    
    let mut client = reqwest::Client::new();

    let mut map = HashMap::new();
    let mut token_to_do = String::new();
    let mut login = String::new();
    let mut group = String::new();
    
    while true {
        showMenu().await;

        token_to_do = "".to_string();
        println!("Enter the program:");
    
        std::io::stdin().read_line(&mut token_to_do).unwrap();
        
        let mut token:&str = "";
        login = "".to_string();
        group = "".to_string();
        if token_to_do.trim_end() == ("member") {
            println!("Enter your login:");
            std::io::stdin().read_line(&mut login).unwrap();
    
            map.insert("login", format!("{}", login.trim_end().to_string()));
            map.insert("groupId", format!("0"));
            map.insert("is_admin", format!(""));
            map.insert("santa_for", format!(""));
    
            token = "newMemb";
        }
        else if token_to_do.trim_end() == ("exit") {
            println!("End the program");
            break;
        }
        else {
            println!("Error: Bad request");
        }
        
        let link = format!("http://{}:{}/{}", address, port, token);
        send_post(link, &mut map,&mut client).await;    
        println!("Enter anything to continue");
        std::io::stdin().read_line(&mut login).unwrap();
    }
    Ok(())
}

async fn showMenu() {
    println!("*****Menu*****");
    println!("1. member - add a new member."); 
    println!("2. group - create a new group.");
    println!("3. join - join to the existing group.");
    println!("4. left - left the group.");
    println!("5. santa - print secret santa.");
    println!("6. exit - end the program.");
    println!("***Admin's menu***"); 
    println!("1. set - set another member as admin.");
    println!("2. resign - resign the admin.");
    println!("3. delete - delete the group.");
    println!("4. start - start the lottery.");
}

async fn send_post(link: String, map: &mut HashMap<&str, String>, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let res = client.post(link)
        .json(&map)
        .send()
        .await?
        .text()
        .await?;
        println!("Response: {}", res);
        Ok(())
}
