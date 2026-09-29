use aes::cipher::{
    block::{BlockCipherDecrypt as _, BlockCipherEncrypt as _},
    Array, KeyInit as _,
};
use delog::hexstr;
use stm32n657_hal::{
    cryp::{self, Cryp},
    rng::Rng,
    saes::{self, Saes},
};

pub fn run(cryp: &mut Cryp, saes: &mut Saes, rng: &mut Rng) {
    if let Err(err) = run_all(cryp, saes, rng) {
        error!("AES: Error: {err}");
    } else {
        info!("AES: OK");
    }
}

fn generate<const N: usize>(rng: &mut Rng) -> [u8; N] {
    const {
        assert!(N.is_multiple_of(4));
    }
    let mut data = [0; N];
    for chunk in data.chunks_exact_mut(4) {
        chunk.copy_from_slice(&rng.next_word().to_be_bytes());
    }
    data
}

fn run_all(cryp: &mut Cryp, saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    run_cryp_aes128(cryp, rng)?;
    run_cryp_aes192(cryp, rng)?;
    run_cryp_aes256(cryp, rng)?;
    run_saes_aes128(saes, rng)?;
    run_saes_aes256(saes, rng)?;
    Ok(())
}

fn run_saes_aes128(saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    info!("SAES with AES-128");

    let key: [_; 16] = generate(rng);
    info!("key = {}", hexstr!(&key));

    let cleartext: [_; 16] = generate(rng);
    let cleartext = Array::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = saes::AesEnc::new(saes, saes::Key::Aes128(key));
    let cipher2 = aes::Aes128Enc::new(&Array::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("SAES encrypt AES-128");
    }

    let cipher1 = saes::AesDec::new(saes, saes::Key::Aes128(key));
    let cipher2 = aes::Aes128Dec::new(&Array::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("SAES decrypt AES-128");
    }

    Ok(())
}

fn run_saes_aes256(saes: &mut Saes, rng: &mut Rng) -> Result<(), &'static str> {
    info!("SAES with AES-256");

    let key: [_; 32] = generate(rng);
    info!("key = {}", hexstr!(&key));

    let cleartext: [_; 16] = generate(rng);
    let cleartext = Array::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = saes::AesEnc::new(saes, saes::Key::Aes256(key));
    let cipher2 = aes::Aes256Enc::new(&Array::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("SAES encrypt AES-256");
    }

    let cipher1 = saes::AesDec::new(saes, saes::Key::Aes256(key));
    let cipher2 = aes::Aes256Dec::new(&Array::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("SAES decrypt AES-256");
    }

    Ok(())
}

fn run_cryp_aes128(cryp: &mut Cryp, rng: &mut Rng) -> Result<(), &'static str> {
    info!("CRYP with AES-128");

    let key: [_; 16] = generate(rng);
    info!("key = {}", hexstr!(&key));

    let cleartext: [_; 16] = generate(rng);
    let cleartext = Array::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = cryp::AesEnc::new(cryp, cryp::Key::Aes128(key));
    let cipher2 = aes::Aes128Enc::new(&Array::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("CRYP encrypt AES-128");
    }

    let cipher1 = cryp::AesDec::new(cryp, cryp::Key::Aes128(key));
    let cipher2 = aes::Aes128Dec::new(&Array::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("CRYP decrypt AES-128");
    }

    Ok(())
}

fn run_cryp_aes192(cryp: &mut Cryp, rng: &mut Rng) -> Result<(), &'static str> {
    info!("CRYP with AES-192");

    let key: [_; 24] = generate(rng);
    info!("key = {}", hexstr!(&key));

    let cleartext: [_; 16] = generate(rng);
    let cleartext = Array::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = cryp::AesEnc::new(cryp, cryp::Key::Aes192(key));
    let cipher2 = aes::Aes192Enc::new(&Array::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("CRYP encrypt AES-192");
    }

    let cipher1 = cryp::AesDec::new(cryp, cryp::Key::Aes192(key));
    let cipher2 = aes::Aes192Dec::new(&Array::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("CRYP decrypt AES-192");
    }

    Ok(())
}

fn run_cryp_aes256(cryp: &mut Cryp, rng: &mut Rng) -> Result<(), &'static str> {
    info!("CRYP with AES-256");

    let key: [_; 32] = generate(rng);
    info!("key = {}", hexstr!(&key));

    let cleartext: [_; 16] = generate(rng);
    let cleartext = Array::from(cleartext);
    info!("cleartext = {}", hexstr!(&cleartext));

    let mut block1 = cleartext;
    let mut block2 = cleartext;

    let cipher1 = cryp::AesEnc::new(cryp, cryp::Key::Aes256(key));
    let cipher2 = aes::Aes256Enc::new(&Array::from(key));
    cipher1.encrypt_block(&mut block1);
    cipher2.encrypt_block(&mut block2);
    info!("ciphertext1 = {}", hexstr!(&block1));
    info!("ciphertext2 = {}", hexstr!(&block2));
    drop(cipher1);

    if block1 != block2 {
        return Err("CRYP encrypt AES-256");
    }

    let cipher1 = cryp::AesDec::new(cryp, cryp::Key::Aes256(key));
    let cipher2 = aes::Aes256Dec::new(&Array::from(key));
    cipher1.decrypt_block(&mut block1);
    cipher2.decrypt_block(&mut block2);
    info!("cleartext1 = {}", hexstr!(&block1));
    info!("cleartext1 = {}", hexstr!(&block2));

    if block1 != block2 {
        return Err("CRYP decrypt AES-256");
    }

    Ok(())
}
