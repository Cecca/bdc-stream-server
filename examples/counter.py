import socket
import time


def main():
    histogram = dict()

    start = time.time()
    cnt = 0
    for x in start_stream(23, 100000):
        cnt += 1
        if x in histogram:
            histogram[x] += 1
        else:
            histogram[x] = 1
    end = time.time()
    print("throughput ", cnt / (end - start))



def read_lines(sock):
    buffer = b''  # Create an empty bytes object to store incoming data
    while True:
        data = sock.recv(1024)  # Receive data from the socket
        if not data:  # If no more data is received, break the loop
            break
        buffer += data  # Append the received data to the buffer
        while b'\n' in buffer:  # Loop through the buffer until a newline character is found
            line, buffer = buffer.split(b'\n', 1)  # Split the buffer at the newline character
            yield line  # Yield the line
    if buffer:  # If there's any remaining data in the buffer after exiting the loop
        yield buffer  # Yield the remaining data



def start_stream(seed, limit):
    address = ("algo.dei.unipd.it", 8888)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    sock.connect(address)
    # seed the generator on the server
    sock.sendall(str(seed).encode('utf-8') + b"\n")

    cnt = 0
    for line in read_lines(sock):
        yield int(line)
        cnt += 1
        if cnt >= limit:
            break

main()

