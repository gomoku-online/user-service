use crate::application::port::outbound::generator::nickname_generator::NicknameGenerator;
use crate::domain::user::user_nickname::UserNickname;
use rand::prelude::IndexedRandom;
use std::fmt::Debug;
use rand::Rng;

const ADJECTIVES: &[&str] = &[
    "현명한",
    "고요한",
    "날렵한",
    "용감한",
    "슬기로운",
    "신중한",
    "빛나는",
    "강력한",
    "평화로운",
    "단단한",
    "즐거운",
    "정확한",
];

const NOUNS: &[&str] = &[
    "기사",
    "바둑알",
    "고수",
    "흑돌",
    "백돌",
    "장인",
    "승부사",
    "호랑이",
    "거북이",
    "학",
    "바람",
    "강물",
    "대나무",
];

#[derive(Debug)]
pub struct NicknameGeneratorImpl;

impl NicknameGenerator for NicknameGeneratorImpl {
    fn generate(&self) -> UserNickname {
        let mut rng = rand::thread_rng();

        let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"슬기로운");

        let noun = NOUNS.choose(&mut rng).unwrap_or(&"바둑알");

        let number: u64 = rng.random_range(1..=999999);

        let nickname_str = format!("{}{}{}", adjective, noun, number);

        UserNickname::new(nickname_str)
    }
}
