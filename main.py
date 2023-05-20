import socket
import time

import clip_c_m

HOST = '127.0.0.1'
PORT = 8080

# initialize clip controller
clip_c = clip_c_m.ClipController()

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.bind((HOST, PORT))
    s.listen()
    print("Python server listening...")
    while True:
        conn, addr = s.accept()
        with conn:
            print(f"Connected by {addr}")
            while True:
                data = conn.recv(4096).decode()
                if not data:
                    break
                now = time.time()
                label, prob = clip_c.get_clip_features(data)
                print(f"Elapsed time: {time.time() - now}")
                print(f"Label: {label}, Probability: {prob}")
                # send probability and label
                conn.sendall(f"{label},{prob}".strip().encode(encoding='utf-8'))
            conn.close()
        print('Connection closed')

