from gevent.pywsgi import WSGIServer
from server import app

http_server = WSGIServer(('', 8000), app)
http_server.serve_forever()
