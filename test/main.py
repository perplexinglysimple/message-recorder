import zmq
import time

# Define the ZeroMQ context and create a PUB socket
context = zmq.Context()
socket = context.socket(zmq.PUB)

# Define the connection address and topic
port = "5557"  # Change this to match the port you want to test
address = f"tcp://localhost:{port}"
topic = "test"  # Use the topic expected by your Rust subscriber

# Connect the socket
socket.bind(address)
print(f"Publishing on {address} with topic '{topic}'")

# Publish messages in a loop
try:
    counter = 0
    while True:
        message = f"{topic} Message {counter}"
        # Send the message with the topic prepended
        socket.send_string(topic, zmq.SNDMORE)
        socket.send_string(message)
        print(f"Sent: {message}")

        counter += 1
        time.sleep(1)  # Wait 1 second between messages
except KeyboardInterrupt:
    print("Publishing stopped.")
finally:
    socket.close()
    context.term()
