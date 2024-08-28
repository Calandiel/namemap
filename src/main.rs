use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

pub mod namemap;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Owie {
    pub key: OwieKey,
}
new_key_type! { pub struct OwieKey; }

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Nowie {
    pub key: NowieKey,
    pub owie: OwieKey,
}
new_key_type! { pub struct NowieKey; }

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Raws {
    pub owies: namemap::Namemap<OwieKey, Owie>,
    pub nowies: namemap::Namemap<NowieKey, Nowie>,
}

fn main() {
    println!("Hello, world!");

    let mut r = Raws::default();
    let test = r.nowies.insert("test".to_string(), Default::default());
    r.nowies.get_mut(test).unwrap().key = test;

    let p = ron::ser::to_string_pretty(&r, ron::ser::PrettyConfig::default()).unwrap();

    println!("BEFORE:");
    println!("{}", p);

    let p = namemap::post_process(p, "{={={".to_string(), "}=}=}".to_string());
    println!("AFTER:");
    println!("{}", p);

    let p = namemap::pre_process(p, "{={={".to_string(), "}=}=}".to_string());
    println!("RE:");
    println!("{}", p);
}
