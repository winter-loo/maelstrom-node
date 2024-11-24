# maelstrom-node

see `git log` for development history

# debugging

```bash
./test.sh serve
```
Then open the URL http://localhost:8080 in your browser and you should see the
messages from the protocol in the messages.svg.

# specification

## Challenge #3b: Multi-Node Broadcast

In this challenge, we simply broadcast a message we have never seen to all other
nodes.

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/01-broadcast.md


## Challenge #3c: Fault Tolerant Broadcast

In this challenge, we need to handle message loss.

reference: https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/02-performance.md

### optimization

First goal: reduce the number of messages sent between nodes(internal servers).
Solution: don't broadcast a message back to the server which sent it to us.