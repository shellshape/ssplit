use anyhow::Result;
use std::{
    collections::HashSet,
    io::{Read, Write},
};

pub enum Splitter {
    Replace(String),
    Indices {
        indices: HashSet<u32>,
        delimiter: String,
    },
}

impl Splitter {
    pub fn try_from_idx(idx: Option<String>, delimiter: String) -> Result<Self> {
        let Some(idx) = idx else {
            return Ok(Self::Replace(delimiter));
        };

        let elems = idx.split(',').map(str::trim);

        let mut indices = HashSet::new();
        for elem in elems {
            if let Some((from, to)) = elem.split_once('-') {
                let from = from.parse()?;
                let to = to.parse()?;
                if from >= to {
                    anyhow::bail!("range begin value must be smaller than end value: {elem}");
                }
                for i in from..=to {
                    indices.insert(i);
                }
            } else {
                indices.insert(elem.parse()?);
            }
        }

        Ok(Self::Indices { indices, delimiter })
    }

    pub fn split_stream<S: Into<String>>(
        &self,
        mut input: impl Read,
        mut output: impl Write,
        split: S,
    ) -> Result<()> {
        let split: String = split.into();

        let mut buf = [0u8; 16 * 1024];
        let mut i = 0;
        let mut first = true;

        loop {
            let n = input.read(&mut buf)?;
            if n == 0 {
                break;
            }

            let str = String::from_utf8(buf[..n].to_vec())?;
            let split = str.split(&split);

            match self {
                Splitter::Replace(with) => {
                    for elem in split.intersperse(with) {
                        output.write_all(elem.as_bytes())?;
                    }
                }
                Splitter::Indices { indices, delimiter } => {
                    for elem in split {
                        if indices.contains(&i) {
                            if !first {
                                output.write_all(delimiter.as_bytes())?;
                            } else {
                                first = false;
                            }
                            output.write_all(elem.as_bytes())?;
                        }
                        i += 1;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, BufWriter};

    #[test]
    fn try_from_index_replace() {
        let delimiter = "\n".to_string();
        let splitter = Splitter::try_from_idx(None, delimiter.clone()).unwrap();
        assert!(matches!(splitter, Splitter::Replace(d) if d == delimiter));
    }

    #[test]
    fn try_from_index_indices() {
        let delimiter = "\n".to_string();

        let splitter = Splitter::try_from_idx(Some("2".into()), delimiter.clone()).unwrap();
        assert!(
            matches!(splitter, Splitter::Indices { indices, delimiter: d } if d == delimiter && 
                indices == HashSet::from([2]))
        );

        let splitter = Splitter::try_from_idx(Some("2, 3,4 ".into()), delimiter.clone()).unwrap();
        assert!(
            matches!(splitter, Splitter::Indices { indices, delimiter: d } if d == delimiter && 
                indices == HashSet::from([2, 3, 4]))
        );

        let splitter = Splitter::try_from_idx(Some("1-3".into()), delimiter.clone()).unwrap();
        assert!(
            matches!(splitter, Splitter::Indices { indices, delimiter: d } if d == delimiter && 
                indices == HashSet::from([1, 2, 3]))
        );

        let splitter =
            Splitter::try_from_idx(Some("1-3,7,8, 9-12 ".into()), delimiter.clone()).unwrap();
        assert!(
            matches!(splitter, Splitter::Indices { indices, delimiter: d } if d == delimiter && 
                indices == HashSet::from([1, 2, 3, 7, 8, 9, 10, 11, 12]))
        );

        let splitter =
            Splitter::try_from_idx(Some("1-3,7,8,7-12 ".into()), delimiter.clone()).unwrap();
        assert!(
            matches!(splitter, Splitter::Indices { indices, delimiter: d } if d == delimiter && 
                indices == HashSet::from([1, 2, 3, 7, 8, 9, 10, 11, 12]))
        );

        let res = Splitter::try_from_idx(Some("1-3,7,8,12-7".into()), delimiter.clone());
        assert!(res.is_err());

        let res = Splitter::try_from_idx(Some("1,2,a".into()), delimiter.clone());
        assert!(res.is_err());

        let res = Splitter::try_from_idx(Some("".into()), delimiter.clone());
        assert!(res.is_err());
    }

    #[test]
    fn splitting_replace() {
        let splitter = Splitter::Replace("\n".into());

        test_split(
            &splitter,
            "hello world what is going on",
            "hello\nworld\nwhat\nis\ngoing\non",
        );
        test_split(&splitter, "hello\nworld", "hello\nworld");
        test_split(&splitter, "", "");
        test_split(&splitter, " ", "\n");
        test_split(&splitter, "  a ", "\n\na\n");
    }

    #[test]
    fn splitting_indices() {
        let splitter = Splitter::Indices {
            delimiter: "\n".into(),
            indices: HashSet::from([1, 3]),
        };

        test_split(&splitter, "hello world what is going on", "world\nis");
        test_split(&splitter, "hello\nworld", "");
        test_split(&splitter, "", "");
        test_split(&splitter, " ", "");
        test_split(&splitter, "   a ", "\na");
    }

    // ----- helpers -----

    fn test_split(splitter: &Splitter, input: &str, expect: &str) {
        let mut input = BufReader::new(input.as_bytes());
        let mut output = BufWriter::new(vec![]);

        splitter.split_stream(&mut input, &mut output, " ").unwrap();

        let res = String::from_utf8(output.buffer().to_vec()).unwrap();
        assert_eq!(res, expect);
    }
}

#[cfg(test)]
mod benches {
    use super::*;

    extern crate test;
    use test::Bencher;
    use test_utils::streams::{RepeatReader, VoidWriter};

    #[bench]
    fn bench_split_replace_1k(b: &mut Bencher) {
        let splitter = Splitter::Replace("\n".into());
        bench_split(b, &splitter, 1024, "Hello world!");
    }

    #[bench]
    fn bench_split_replace_1m(b: &mut Bencher) {
        let splitter = Splitter::Replace("\n".into());
        bench_split(b, &splitter, 1024 * 1024, "Hello world!");
    }

    // ----- helper -----

    fn bench_split(b: &mut Bencher, splitter: &Splitter, size: usize, content: &str) {
        let mut input = RepeatReader::from_str(size, content);
        let mut output = VoidWriter::new();

        b.iter(|| {
            let _ = splitter.split_stream(&mut input, &mut output, " ");
            let _ = input.reset();
        });
    }
}
