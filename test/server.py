from http.server import BaseHTTPRequestHandler, HTTPServer
import json


class AuthHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == "/user/auth":
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            print("req")
            print(post_data)
            request_data = json.loads(post_data)
            token = "x" * 32
            response = {
                "token": token,
                "email": request_data["email"],
                "level": 1
            }
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode('utf-8'))
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")

server_address = ('', 5432)
httpd = HTTPServer(server_address, AuthHandler)
httpd.serve_forever()