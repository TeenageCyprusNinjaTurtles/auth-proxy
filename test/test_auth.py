from http.server import BaseHTTPRequestHandler, HTTPServer
import json
from time import sleep
import requests
import faker
import threading

import logging
logging.basicConfig(level=logging.DEBUG)

faker = faker.Faker()

class AuthHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == "/user/auth":
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            print("req")
            print(post_data)
            request_data = json.loads(post_data)
            token = "x" * 32
            if request_data["email"] == "wrong@data.com":
                response = {
                    "result": "SOMETHING_WRONG"
                }
            else:
                response = {
                    "token": token,
                    "email": request_data["email"],
                    "level": 1
                }
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode('utf-8'))
        elif self.path == "/user/test":
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"result": "OK"}).encode('utf-8'))
        elif self.path == "/training/get":
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            print("req")
            print(post_data)
            # request_data = json.loads(post_data)
            response = {
                "level": self.headers["X-User-Level"],
                "email": self.headers["X-User-Email"]
            }
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode('utf-8'))           
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")

def run_auth_server():
    server_address = ('', 5432)
    httpd = HTTPServer(server_address, AuthHandler)
    httpd.serve_forever()

server_thread = threading.Thread(target=run_auth_server)
server_thread.daemon = True
server_thread.start()



def test_token():
    email = faker.email()
    response = requests.post("http://localhost:8081/user/auth", json={
        "email": email,
        "password": faker.password()
    })
    assert response.status_code == 200
    print(response.json())
    response_data = response.json()
    response_data["token"] == "x" * 32
    assert type(response_data["expiration"]) == int


    response = requests.post("http://localhost:8081/user/auth", json={
        "email": "wrong@data.com",
        "password": faker.password()
    })

    assert response.json() == {"result": "SOMETHING_WRONG"}
    assert response.status_code == 200

    response = requests.post("http://localhost:8081/user/auth", json={
        "email": email,
        "password": faker.password()
    })
    token = response.json()["token"]
    response = requests.post("http://localhost:8081/user/test", headers={
    })
    assert response.json() == {"result": "OK"}

    response = requests.post("http://localhost:8081/user/test", headers={
        'X-Auth-Token': 'WRONG_TOKEN'
    })
    assert response.json() == {"result": "ACCESS_ERROR"}

    response = requests.post("http://localhost:8081/user/test", headers={
        'X-Auth-Token': token
    })
    assert response.json() == {"result": "OK"}

    response = requests.post("http://localhost:8081/training/get", headers={
    })
    assert response.json() == {"result": "ACCESS_ERROR"}

    response = requests.post("http://localhost:8081/training/get", headers={
        'X-Auth-Token': token
    })
    assert response.json()['email'] == email
    assert response.json()['level'] == '1'


    print("test_token passed")
    
test_token()