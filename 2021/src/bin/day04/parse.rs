#[test]
fn perry_parse() -> anyhow::Result<()> {
    use std::io;
    use std::io::prelude::*;

    let input = r#"
76 82  2 92 53
74 33  8 89  3
80 27 72 26 91
30 83  7 16  4
20 56 48  5 13

0  82  2 92 53
74 33  8 89  3
80 27 72 26 91
30 83  7 16  4
20 56 48  5 13
"#;

    let reader = io::BufReader::new(input.as_bytes());
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let mut iter = lines.iter();

    // skip blank line
    iter.next().ok_or(anyhow::anyhow!("no blank line?"))?;

    // Use `by_ref` so we can use iter again after we're done
    // parsing this board.
    let board: Vec<Vec<u32>> = iter
        .by_ref()
        .take(5)
        .map(|line| {
            line.split_whitespace()
                .map(str::parse)
                .collect()
        })
        .collect::<Result<_, _>>()?;

    // skip blank line
    iter.next().ok_or(anyhow::anyhow!("no blank line?"))?;

    let board2: Vec<Vec<u32>> = iter
        .by_ref()
        .take(5)
        .map(|line| {
            line.split_whitespace()
                .map(str::parse)
                .collect()
        })
        .collect::<Result<_, _>>()?;

    assert_eq!(board,
               vec![
                   vec![76, 82,  2, 92, 53],
                   vec![74, 33,  8, 89,  3],
                   vec![80, 27, 72, 26, 91],
                   vec![30, 83,  7, 16,  4],
                   vec![20, 56, 48,  5, 13],
               ]);

    assert_eq!(board2,
               vec![
                   vec![0,  82,  2, 92, 53],
                   vec![74, 33,  8, 89,  3],
                   vec![80, 27, 72, 26, 91],
                   vec![30, 83,  7, 16,  4],
                   vec![20, 56, 48,  5, 13],
               ]);

    Ok(())
}
