# maelstrom-node

see `git log` for development history

# debugging tips

## serve

```bash
./test.sh serve
```

Then open the URL http://localhost:8080 in your browser and you should see the
messages from the protocol in the messages.svg.

## log

Use `eprintln!`. The output will be logged to, such as `store/broadcast/latest/node-logs/n0.log`.


# specification

## Challenge #3b: Multi-Node Broadcast

In this challenge, we simply broadcast a message we have never seen to all other
nodes.

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/01-broadcast.md


## Challenge #3c: Fault Tolerant Broadcast

In this challenge, we need to handle message loss.

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/02-performance.md

### optimization

First goal: reduce the number of messages sent between nodes(internal servers).
Solution: don't broadcast a message back to the server which sent it to us.

### dealing with failure

when messages are lost, we need to retry sending the message and we need a way to know when to stop retrying.

## Challenge #7a: Datomic Transactor Model

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/05-datomic/01-single-node.md

## Challenge #7b: Shared State

In this challenge, we store entire database in memory.

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/05-datomic/02-shared-state.md


## Challenge #7c: persistent trees

In this challenge, we store the pair (k, id) instead of (k, values) and the id
points to the values.

In the first step, we refacot the code to use `Rc<RefCell<Node>>`.

In the second step, we use **composition** and **generics** to structure the code instead of inheritance used in Ruby.

reference:
  * https://github.com/jepsen-io/maelstrom/blob/main/doc/05-datomic/03-persistent-trees.md
  * https://www.thecodedmessage.com/posts/oop-3-inheritance/
  * https://winter-loo.notion.site/code-reuse-151763aede9c8029a6bccfe7517d0975