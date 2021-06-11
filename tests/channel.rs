use std::{thread, time::Duration};

use live_coding_channel::Channel;

#[test]
fn channel_send_recv() {
    let chan = Channel::new(1);
    chan.try_send(1).unwrap();
    assert_eq!(chan.try_recv(), Some(1));
}

#[test]
fn channel_clone_receives_data() {
    let chan1 = Channel::new(1);
    let chan2 = chan1.clone();

    chan1.try_send('a').unwrap();
    assert_eq!(chan2.try_recv(), Some('a'));
}

#[test]
fn mpmc() {
    let chan1 = Channel::new(2);
    let chan2 = chan1.clone();

    let chan3 = chan1.clone();
    let chan4 = chan1.clone();

    chan1.try_send("1->3").unwrap();
    chan2.try_send("2->4").unwrap();
    assert_eq!(chan3.try_recv(), Some("1->3"));
    assert_eq!(chan4.try_recv(), Some("2->4"));
}

#[test]
fn multithreaded() {
    let chan = Channel::new(1);

    let sender_thread = thread::spawn({
        let sender = chan.clone();
        move || {
            thread::sleep(Duration::from_millis(2));
            sender.try_send("from another thread")
        }
    });

    assert_eq!(chan.try_recv(), None);

    let join = sender_thread.join();
    let send_result = join.unwrap();
    send_result.unwrap();

    assert_eq!(chan.try_recv(), Some("from another thread"));
}
