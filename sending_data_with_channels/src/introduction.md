# Introduction

Today, we're going to be talking about *channels*. Channels are a pretty old idea. Van Jacobson came up with the idea in 2006 as a quick way to move data between parts of the network stack. Van Jacobson's channels were quite UNIX specific, focusing on a lock-free ringbuffer with concurrent reading and writing as a way to rapidly move data around. Modern channels have changed quite a bit, but the basic idea remains.

> Channels let you move data from point A to point B. Point A and B may be on different threads, different cores, or different async tasks.

## Go Channels

The Go language is all about channels. Go encourages you to spawn thousands of tasks ("goroutines" - a pun on coroutines), and send data between them with channels. Go makes them relatively easy to use (with a few footguns, such as forgetting to close a channel and leaking memory, or writing to a closed channel and crashing your whole runtime).

Go also provides an infrastructure that makes channels easy. Go doesn't give you a lot of control over your runtime environment, ensuring that you have a relatively Erlang-like:

* One thread per core (this is configurable).
* Each thread runs an async runtime.
* Each thread can steal work from other threads, so if an async task is taking too long it will execute on another core.

This is called *green threading*---and makes life very easy for channels. Data winds up in a garbage-collected heap, so ownership is hidden from the user.

## Rust Channels

Rust's promise of "fearless concurrency" includes channels as part of the infrastructure. Unlike Go, Rust isn't opinionated about how you structure your program. So channels in Rust have to obey the borrow checker, remain data-race safe, and work with the RAII---Resource Acquisition is Initialization---framework provided by the `Drop` trait. Rust 1.0 also had several years of Go experience with crashing to build upon!

### Threaded Channels

The standard library ships with thread-oriented channels. Unlike Go, you have to make your own threads---but you can connect them together with channels with very little effort.

The most basic channel is the "MPSC" channel---Multi Producer, Single Consumer. You can send from as many event sources as you want, and all the events wind up at a single recipient. Many crates exist that provide other channel types---broadcast (one to all), multi-producer-multi-consumer (a big mesh, yes I pronounced that incorrectly as "mess" on purpose), single-producer-single-consumer (1:1), oneshot (send a single message only) and even "reply channels" that setup bidirectional communication.

### Async Channels

Rust *also* provides async channels. In fact, if you use Tokio in its default multi-threaded mode it basically reimplimenets Go for for you:

* One thread per core (this is configurable).
* Each thread runs an async runtime.
* Each thread can steal work from other threads, so if an async task is taking too long it will execute on another core.
* Async-friendly channels for firing events between tasks.

So we're going to learn how to make channels work for us---and try to have our cake and eat it, using tiny amounts of RAM, performing at blazing speeds, and enjoying life without data races!
