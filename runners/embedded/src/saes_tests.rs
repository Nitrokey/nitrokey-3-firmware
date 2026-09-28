use aes::cipher::{
    generic_array::GenericArray, BlockDecrypt as _, BlockEncrypt as _, KeyInit as _,
};
use delog::hexstr;
use stm32n657_hal::{
    rng::Rng,
    saes::{self, Key, Saes},
};

pub fn run(saes: &mut Saes, rng: &mut Rng) {
    if let Err(err) = run_all(saes, rng) {
        error!("SAES: Error: {err}");
    } else {
        info!("SAES: OK");
    }
}

fn run_all(saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    run_aes128(saes, rng)?;
    run_aes256(saes, rng)?;
    Ok(())
}

fn run_aes128(saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    info!("AES-128");

    let mut key = [0; 16];
    for i in 0..4 {
        key[i * 4..][..4].copy_from_slice(&rng.next_word().to_be_bytes());
    }
    info!("key = {}", hexstr!(&key));

    let mut cleartext = [0; 16];
    for i in 0..4 {
        cleartext[i * 4..][..4].copy_from_slice(&rng.next_word().to_be_bytes());
    }
    let cleartext = GenericArray::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = saes::AesEnc::new(saes, Key::Aes128(key));
    let cipher2 = aes::Aes128Enc::new(&GenericArray::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("encrypt AES-128");
    }

    let cipher1 = saes::AesDec::new(saes, Key::Aes128(key));
    let cipher2 = aes::Aes128Dec::new(&GenericArray::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("decrypt AES-128");
    }

    Ok(())
}

fn run_aes256(saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    info!("AES-256");

    let mut key = [0; 32];
    for i in 0..8 {
        key[i * 4..][..4].copy_from_slice(&rng.next_word().to_be_bytes());
    }
    info!("key = {}", hexstr!(&key));

    let mut cleartext = [0; 16];
    for i in 0..4 {
        cleartext[i * 4..][..4].copy_from_slice(&rng.next_word().to_be_bytes());
    }
    let cleartext = GenericArray::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = saes::AesEnc::new(saes, Key::Aes256(key));
    let cipher2 = aes::Aes256Enc::new(&GenericArray::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("encrypt AES-256");
    }

    let cipher1 = saes::AesDec::new(saes, Key::Aes256(key));
    let cipher2 = aes::Aes256Dec::new(&GenericArray::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("decrypt AES-256");
    }

    Ok(())
}
