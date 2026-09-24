use pco::{ChunkConfig, standalone::simple_compress};

use super::*;

#[test]
fn counts_and_section_termination_are_exact() {
    for count in [0, 1, 255, 256, 257, 1000] {
        let values = (0..count as u32).collect::<Vec<_>>();
        let encoded = simple_compress(&values, &ChunkConfig::default()).unwrap();
        assert_eq!(exact::<u32>(&encoded, count).unwrap(), values);
        assert!(exact::<u32>(&encoded, count + 1).is_err());
        if count > 0 {
            assert!(exact::<u32>(&encoded, count - 1).is_err());
        }
        let mut trailing = encoded;
        trailing.push(0);
        assert!(exact::<u32>(&trailing, count).is_err());
    }
}
