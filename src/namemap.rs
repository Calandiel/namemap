use regex::Regex;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use slotmap::{Key, SlotMap};

const BM: &str = "|!@#|ORIGIN|%%%|1015122487|#@!|";
const FMID: &str = "|!@#|SEPARATOR|%%%|56461790|#@!|";
const SMID: &str = "|!@#|1036240365|%%%|SEPARATOR|#@!|";
const EM: &str = "|!@#|1513832715|%%%|END|#@!|";
const DEFAULT_KEY: &str = "(idx:4294967295,version:1)";
const DEFAULT_NAME: &str = "default";

/// Call this after serialization to ron
pub fn pre_process(mut ron_string: String, opener: String, finisher: String) -> String {
    let mut counter = 0;
    let mut get_next_unique_vv = || {
        counter += 1;
        format!("(idx:{},version:1)", counter)
    };

    let mut finds: Vec<(String, String)> = vec![];
    finds.push((DEFAULT_NAME.to_string(), DEFAULT_KEY.to_string()));
    // First, fetch all existing strings keys/names uwu
    loop {
        let ope = "\"".to_string() + opener.as_str();
        let pos_op = ron_string.find(ope.as_str());
        if pos_op.is_none() {
            break;
        }
        let pos_op = pos_op.unwrap();

        let fin = finisher.clone() + "\"";
        let pos_fi = ron_string.find(fin.as_str());
        if pos_fi.is_none() {
            break;
        }
        let pos_fi = pos_fi.unwrap();

        finds.push((
            ron_string[pos_op + ope.len()..pos_fi].to_string(),
            get_next_unique_vv(),
        ));

        ron_string = ron_string[..pos_op].to_string()
            + "\""
            + &ron_string[pos_op + ope.len()..pos_fi]
            + "\""
            + &ron_string[pos_fi + fin.len()..];
    }

    for (name, key) in finds {
        let searcher = opener.clone() + name.as_str() + finisher.as_str();
        ron_string = ron_string.replace(searcher.as_str(), key.as_str());
    }
    // After everything is fetched, we can commence the great replacement!
    ron_string
}

/// Call this after serialization to ron
pub fn post_process(mut ron_string: String, opener: String, finisher: String) -> String {
    let mut stored_keys: Vec<(String, String)> = vec![];
    stored_keys.push((DEFAULT_KEY.to_string(), DEFAULT_NAME.to_string()));

    loop {
        let pos_bm = ron_string.find(BM);
        if pos_bm.is_none() {
            break;
        }
        let pos_bm = pos_bm.unwrap();

        let pos_fmid = ron_string.find(FMID);
        if pos_fmid.is_none() {
            break;
        }
        let pos_fmid = pos_fmid.unwrap();

        let pos_smid = ron_string.find(SMID);
        if pos_smid.is_none() {
            break;
        }
        let pos_smid = pos_smid.unwrap();

        let pos_em = ron_string.find(EM);
        if pos_em.is_none() {
            break;
        }
        let pos_em = pos_em.unwrap();

        let key_ron = ron_string[pos_fmid + FMID.len()..pos_smid].to_string();
        let default_key_ron = ron_string[pos_smid + SMID.len()..pos_em].to_string();
        let name = ron_string[pos_bm + BM.len()..pos_fmid].to_string();
        assert!(DEFAULT_KEY == default_key_ron, "The default key from serialization should be the same as the default key from the data type.");

        // println!("FOUND KEY: {} :: {}", key_ron, default_key_ron);
        let cloned_name = name.clone();
        let duplicate_name = stored_keys
            .iter()
            .position(|(_, mapped_name)| *mapped_name == cloned_name);
        let duplicate_key = stored_keys.iter().position(|(key, _)| *key == key_ron);
        if duplicate_name.is_some() {
            panic!("Duplicate name: {}", cloned_name);
        }
        if duplicate_key.is_some() {
            panic!("Duplicate key: {}", key_ron);
        }

        stored_keys.push((key_ron.clone(), name.clone()));

        ron_string = ron_string[..pos_bm].to_string()
            + opener.as_str()
            + name.as_str()
            + finisher.as_str()
            + &ron_string[(pos_em + EM.len())..];
    }

    for (key, name) in stored_keys {
        let mut regex_matcher = "".to_string();
        let mut cs = key.chars();
        let ca = cs.next().unwrap();

        let push_one = |regex_matcher: &mut String, ca: char| {
            regex_matcher.push_str(
                if ca == '(' || ca == ')' {
                    "\\".to_string() + ca.to_string().as_str()
                } else {
                    ca.to_string()
                }
                .as_str(),
            );
        };
        push_one(&mut regex_matcher, ca);
        for c in cs {
            regex_matcher.push_str("[\\s,]*");
            push_one(&mut regex_matcher, c);
        }
        // println!("REGEX: |{}| -> {}", regex_matcher, name);
        let reg = Regex::new(regex_matcher.as_str()).unwrap().replace_all(
            &ron_string,
            opener.clone() + name.as_str() + finisher.as_str(),
        );
        ron_string = reg.to_string();
    }
    ron_string
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Namemap<K, V>
where
    K: Key,
{
    data: SlotMap<K, V>,
    ids_to_keys: std::collections::HashMap<String, K>,
    keys_to_ids: std::collections::HashMap<K, String>,
}

impl<K, V> Serialize for Namemap<K, V>
where
    K: Key + Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        assert!(
            self.ids_to_keys.len() == self.data.len(),
            "The number of stored keys should be the same as the slotmap size.",
        );
        assert!(
            self.keys_to_ids.len() == self.data.len(),
            "The number of stored ids should be the same as the slotmap size.",
        );
        let mut map = serializer.serialize_map(Some(self.ids_to_keys.len()))?;
        for (k, v) in &self.data {
            let ron_key = ron::ser::to_string(&k).unwrap();
            let ron_key_default = ron::ser::to_string(&K::default()).unwrap();
            let id = self.keys_to_ids.get(&k).unwrap(); // should never fail
            let id = BM.to_string()
                + id.as_str()
                + FMID
                + ron_key.as_str()
                + SMID
                + ron_key_default.as_str()
                + EM;
            map.serialize_entry(&id, v)?;
        }
        map.end()
    }
}
/*
impl<'de> Deserialize<'de> for i32 {
    fn deserialize<D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i32(I32Visitor)
    }
}
*/

impl<'a, K, V> Namemap<K, V>
where
    K: Key,
{
    pub fn insert(&mut self, id: String, v: V) -> K {
        assert!(
            !self.ids_to_keys.contains_key(&id),
            "A key should not already exist in the namemap when its inserted"
        );
        let k = self.data.insert(v);
        self.ids_to_keys.insert(id.clone(), k);
        self.keys_to_ids.insert(k, id);
        k
    }

    pub fn remove(&mut self, k: K) -> Option<V> {
        if let Some(v) = self.data.remove(k) {
            let id = self.keys_to_ids.remove(&k).unwrap();
            let _ = self.ids_to_keys.remove(&id).unwrap();
            Some(v)
        } else {
            None
        }
    }

    pub fn get_key(&self, id: String) -> Option<K> {
        self.ids_to_keys.get(&id).copied()
    }

    pub fn get_id(&self, k: K) -> Option<String> {
        self.keys_to_ids.get(&k).cloned()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.data.get_mut(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&'a self) -> slotmap::basic::Iter<'a, K, V> {
        let i = self.data.iter();
        i
    }
}
