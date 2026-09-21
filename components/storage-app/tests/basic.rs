pub mod protocol;
pub mod utils;

use storage_app::Error;

use protocol::Status;

#[test]
fn test_status() {
    let pin = b"123456";
    let bad_pin = b"deadbeef";

    utils::with_app(|app, _| {
        let status = protocol::status(app).unwrap();
        assert_eq!(
            status,
            Status {
                unlocked: false,
                pin_set: false,
                pin_retries: 3,
            }
        );

        protocol::set_pin(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(
            status,
            Status {
                unlocked: false,
                pin_set: true,
                pin_retries: 3,
            }
        );

        protocol::unlock(app, bad_pin).err().unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(
            status,
            Status {
                unlocked: false,
                pin_set: true,
                pin_retries: 2,
            }
        );

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(
            status,
            Status {
                unlocked: true,
                pin_set: true,
                pin_retries: 3,
            }
        );

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(
            status,
            Status {
                unlocked: false,
                pin_set: true,
                pin_retries: 3,
            }
        );
    })
}

#[test]
fn test_pin_retries() {
    let pin = b"123456";
    let bad_pin = b"deadbeef";

    utils::with_app(|app, _| {
        let status = protocol::status(app).unwrap();
        assert_eq!(status.pin_retries, 3);

        protocol::set_pin(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(status.pin_retries, 3);

        for i in 0..2 {
            let result = protocol::unlock(app, bad_pin);
            assert_eq!(result, Err(Error::InvalidPin));
            let status = protocol::status(app).unwrap();
            assert_eq!(status.pin_retries, 2 - i);
        }

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert_eq!(status.pin_retries, 3);

        for i in 0..3 {
            let result = protocol::unlock(app, bad_pin);
            assert_eq!(result, Err(Error::InvalidPin));
            let status = protocol::status(app).unwrap();
            assert_eq!(status.pin_retries, 2 - i);
        }

        let result = protocol::unlock(app, pin);
        assert_eq!(result, Err(Error::PinBlocked));
        let status = protocol::status(app).unwrap();
        assert_eq!(status.pin_retries, 0);
    })
}

#[test]
fn test_set_pin() {
    utils::with_app(|app, _| {
        let pin = b"123456";
        for n in 0..6 {
            let pin = &pin[..n];
            let result = protocol::set_pin(app, pin);
            assert_eq!(result, Err(Error::PinTooShort));
            let status = protocol::status(app).unwrap();
            assert!(!status.pin_set);
        }

        for n in [129, 255, 512] {
            let pin = vec![0xff; n];
            let result = protocol::set_pin(app, &pin);
            assert_eq!(result, Err(Error::PinTooLong));
            let status = protocol::status(app).unwrap();
            assert!(!status.pin_set);
        }

        protocol::set_pin(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);

        let result = protocol::set_pin(app, pin);
        assert_eq!(result, Err(Error::PinAlreadySet));
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
    })
}

#[test]
fn test_change_pin() {
    let pin1 = b"123456";
    let pin2 = b"12345678";

    utils::with_app(|app, data| {
        let result = protocol::change_pin(app, pin1, pin2);
        assert_eq!(result, Err(Error::PinNotSet));
        let status = protocol::status(app).unwrap();
        assert!(!status.pin_set);

        assert_eq!(data.borrow().init_count, 0);
        protocol::set_pin(app, pin1).unwrap();
        assert_eq!(data.borrow().init_count, 1);
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);

        protocol::change_pin(app, pin1, pin2).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
        assert_eq!(status.pin_retries, 3);

        let result = protocol::change_pin(app, pin1, pin2);
        assert_eq!(result, Err(Error::InvalidPin));
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
        assert_eq!(status.pin_retries, 2);

        protocol::change_pin(app, pin2, pin1).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
        assert_eq!(status.pin_retries, 3);

        for n in 0..6 {
            let pin = &pin1[..n];
            let result = protocol::change_pin(app, pin1, pin);
            assert_eq!(result, Err(Error::PinTooShort));
            let status = protocol::status(app).unwrap();
            assert!(status.pin_set);
            assert_eq!(status.pin_retries, 3);
        }

        for n in [129, 255, 512] {
            let pin = vec![0xff; n];
            let result = protocol::change_pin(app, pin1, &pin);
            assert_eq!(result, Err(Error::PinTooLong));
            let status = protocol::status(app).unwrap();
            assert!(status.pin_set);
            assert_eq!(status.pin_retries, 3);
        }

        for i in 0..2 {
            let result = protocol::change_pin(app, pin2, pin1);
            assert_eq!(result, Err(Error::InvalidPin));
            let status = protocol::status(app).unwrap();
            assert!(status.pin_set);
            assert_eq!(status.pin_retries, 2 - i);
        }

        protocol::change_pin(app, pin1, pin2).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
        assert_eq!(status.pin_retries, 3);

        for i in 0..3 {
            let result = protocol::change_pin(app, pin1, pin2);
            assert_eq!(result, Err(Error::InvalidPin));
            let status = protocol::status(app).unwrap();
            assert!(status.pin_set);
            assert_eq!(status.pin_retries, 2 - i);
        }

        let result = protocol::change_pin(app, pin2, pin1);
        assert_eq!(result, Err(Error::PinBlocked));
        let status = protocol::status(app).unwrap();
        assert!(status.pin_set);
        assert_eq!(status.pin_retries, 0);

        assert_eq!(data.borrow().init_count, 1);
    })
}

#[test]
fn test_lock() {
    let pin = b"123456";

    utils::with_app(|app, data| {
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 0);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 0);

        protocol::set_pin(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 0);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 0);

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.unlocked);
        assert!(data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 0);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 1);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().lock_count, 1);
    })
}

#[test]
fn test_unlock() {
    let pin = b"123456";
    let bad_pin = b"deadbeef";

    utils::with_app(|app, data| {
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 0);

        let result = protocol::unlock(app, pin);
        assert_eq!(result, Err(Error::PinNotSet));
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 0);

        protocol::set_pin(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 0);

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.unlocked);
        assert!(data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 1);

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.unlocked);
        assert!(data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 1);

        let result = protocol::unlock(app, bad_pin);
        assert_eq!(result, Err(Error::InvalidPin));
        let status = protocol::status(app).unwrap();
        assert!(status.unlocked);
        assert!(data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 1);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 1);

        let result = protocol::unlock(app, bad_pin);
        assert_eq!(result, Err(Error::InvalidPin));
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 1);

        protocol::unlock(app, pin).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(status.unlocked);
        assert!(data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 2);

        protocol::lock(app).unwrap();
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 2);

        for _ in 0..3 {
            let result = protocol::unlock(app, bad_pin);
            assert_eq!(result, Err(Error::InvalidPin));
            let status = protocol::status(app).unwrap();
            assert!(!status.unlocked);
            assert!(!data.borrow().is_unlocked);
            assert_eq!(data.borrow().unlock_count, 2);
        }

        let result = protocol::unlock(app, bad_pin);
        assert_eq!(result, Err(Error::PinBlocked));
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 2);

        let result = protocol::unlock(app, pin);
        assert_eq!(result, Err(Error::PinBlocked));
        let status = protocol::status(app).unwrap();
        assert!(!status.unlocked);
        assert!(!data.borrow().is_unlocked);
        assert_eq!(data.borrow().unlock_count, 2);
    })
}
