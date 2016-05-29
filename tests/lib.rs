#[macro_use]
extern crate bytevec;

use bytevec::{ByteEncodable, ByteDecodable};

#[test]
fn test_serialize_vec() {
    bytevec_impls! {
        #[derive(PartialEq, Eq, Debug)]
        struct Employee {
            id: u32,
            profile: Profile,
            dept: String
        }
        
        #[derive(PartialEq, Eq, Debug)]
        struct Profile {
            id: u32,
            name: String,
            last_name: String
        }
    }
    
    let employees_1 = vec![
        Employee {
            id: 1,
            profile: Profile {
                id: 10000,
                name: "Michael".to_string(),
                last_name: "Jackson".to_string()
            },
            dept: "music".to_string()
        },
        Employee {
            id: 2,
            profile: Profile {
                id: 10001,
                name: "John".to_string(),
                last_name: "Cena".to_string()
            },
            dept: "wrestling".to_string()
        }
    ];
    let bytes = employees_1.encode().unwrap();
    let employees_2 = Vec::<Employee>::decode(&bytes).unwrap();
    assert_eq!(employees_1, employees_2);
}


bytevec_impls! {
    #[derive(PartialEq, Eq, Debug)]
    struct MeetingsLog {
        id: u32,
        owner: String,
        meetings: std::collections::HashMap<String, String>
    }
}

#[test]
fn test_serialize_slices() {
    let slice = &['1', '2', '3'];
    let bytes = slice.encode().unwrap();
    let vec = Vec::<char>::decode(&bytes).unwrap();
    assert_eq!(vec, slice);
}

#[test]
fn test_serialize_set() {
    let mut set_1 = std::collections::HashSet::new();
    set_1.insert(("One!".to_string(), 1u32));
    set_1.insert(("Two!".to_string(), 2));
    set_1.insert(("Three!".to_string(), 3));
    let bytes = set_1.encode().unwrap();
    let set_2 = std::collections::HashSet::decode(&bytes).unwrap();
    assert_eq!(set_1, set_2);
}

#[test]
fn test_serialize_map() {
    let mut classes_1 = std::collections::HashMap::new();
    classes_1.insert(101usize, "Programming 1".to_string());
    classes_1.insert(102, "Basic CS".to_string());
    let bytes = classes_1.encode().unwrap();
    let classes_2 = std::collections::HashMap::decode(&bytes).unwrap();
    assert_eq!(classes_1, classes_2);
}

#[test]
fn test_serialize_slice_with_map_containers() {
    let slice = &[
        MeetingsLog {
            id: 1,
            owner: "Jack".to_string(),
            meetings: {
                let mut meetings_jack = std::collections::HashMap::new();
                meetings_jack.insert("New York".to_string(), "Michael".to_string());
                meetings_jack.insert("Nippon".to_string(), "Koichi".to_string());
                meetings_jack
            }
        },
        MeetingsLog {
            id: 2,
            owner: "Juan".to_string(),
            meetings: {
                let mut meeting_juan = std::collections::HashMap::new();
                meeting_juan.insert("España".to_string(), "José".to_string());
                meeting_juan.insert("Korea".to_string(), "Lee Hyun".to_string());
                meeting_juan
            }
        }
    ];
     
    let bytes = slice.encode().unwrap();
    let vec = Vec::<MeetingsLog>::decode(&bytes).unwrap();
    assert_eq!(vec, slice);
}