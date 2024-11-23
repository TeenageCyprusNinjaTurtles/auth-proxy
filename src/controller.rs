use std::{collections::HashMap, io::Read};

use rouille::{Request, RequestBody, Response};
use r2d2::Pool;
use crate::{lib::{self, utils}, service_map::ServiceMap, AuthExternalResponse, AuthInternalResponse, ErrorResponse, UserObject};

pub fn get_guest_user_object() -> UserObject {
    UserObject {
        email: "".to_string(),
        level: 0,
    }
}

pub fn get_user_object(request: &Request, pool: Pool<redis::Client>) -> UserObject {
    let token = utils::get_header_value(request, "X-Auth-Token");
    if token.is_none() {
        return get_guest_user_object();
    }
    let mut con = pool.get().unwrap();
    let key = format!("token:{}", token.unwrap());
    let result: Result<String, redis::RedisError> = redis::cmd("GET").arg(key.clone()).query(&mut con);
    if result.is_err() {
        return     UserObject {
            email: "".to_string(),
            level: -1,
        }
    } else {
        let result = result.unwrap();
        let splited_params: Vec<&str> = result.split(":").collect();
        if splited_params.len() != 3 {
            log::error!("Invalid token response format");
            return get_guest_user_object();
        }
        let email = utils::from_b64(splited_params[0]);
        let level = splited_params[1].parse::<i32>().unwrap();
        let expiration = splited_params[2].parse::<u64>().unwrap();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if current_time > expiration {
            redis::cmd("DEL").arg(key).exec(&mut con).unwrap();
            return get_guest_user_object();
        }

        return UserObject {
            email: email,
            level: level,
        };
    }

}

pub fn on_auth(auth_body: &AuthInternalResponse, pool: Pool<redis::Client>) -> Response {
    let mut con = pool.get().unwrap();
    let key = format!("token:{}", auth_body.token.clone());
    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + (3600*24);

    let email = utils::to_b64(auth_body.email.as_str());

    let payload = format!("{}:{}:{}", email, auth_body.level.clone(), expiration);
    let result = redis::cmd("SET").arg(key).arg(payload).exec(&mut con);
    if result.is_err() {
        return Response::text("DB Error").with_status_code(500);
    }
    return Response::json(&AuthExternalResponse {
        token: auth_body.token.clone(),
        expiration: expiration,
    });
}


pub fn pass_by(request: &Request, user_object: &UserObject, service_map: &ServiceMap, body: &Vec<u8>) -> Response {
    
    let user_level = user_object.level.to_string();
    
    let url = request.url();
    let entrypoint = url.split("/").collect::<Vec<&str>>()[1];
    let service_data = service_map.get_service(entrypoint);
    println!("{:?}", service_data);
    if service_data.is_none() {
        return Response::text("Service not found").with_status_code(404);
    }
    let service_data = service_data.unwrap();

    if service_data.auth_only && user_object.level == 0 {
        log::info!("Access error due to authorization level");
        return Response::json(&ErrorResponse{result: lib::enums::ERROR_RESPONSE_ACCESS_ERROR.to_string()});
    }

    let addr = format!("http://{}:{}{}", service_data.host, service_data.port, url);
    let response_body = ureq::post(addr.as_str())
        .set("X-User-Email", user_object.email.as_str())
        .set("X-User-Level", user_level.as_str())
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(2));
    
    
    let res = response_body.send_bytes(&body);
    if res.is_err() {
        return Response::text("Service not available").with_status_code(500);
    }
    let res = res.unwrap().into_string().unwrap();
    

    let response = Response::from_data("application/json", res);
    return response;
}