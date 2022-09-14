import requests

from threading import Thread

# send request to http://127.0.0.1:8000
def send_request(host, port):
    for _ in range(100):
        r = requests.post(f"http://{host}:{port}")
        print(f"Receive response: '{r.text}' from {r.url}")


if __name__ == '__main__':
    t_lst = []
    for _ in range(4):
        t = Thread(target=send_request, args=('127.0.0.1', 8000))
        t_lst.append(t)
        t.start()

    for t in t_lst:
        t.join()