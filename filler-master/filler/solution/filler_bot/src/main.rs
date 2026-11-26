use std::collections::VecDeque;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<char>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<(usize, usize)>,
}

pub fn symbols(player: u8) -> (&'static [char], &'static [char]) {
    if player == 1 {
        (&['@', 'a'], &['$', 's'])
    } else {
        (&['$', 's'], &['@', 'a'])
    }
}

fn read_line<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut s = String::new();
    match reader.read_line(&mut s) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(s.trim_end_matches(['\n', '\r']).to_string()),
    }
}

pub fn parse_dims(line: &str) -> Option<(usize, usize)> {
    let mut it = line.trim().trim_end_matches(':').split_whitespace();
    it.next()?;
    let w = it.next()?.parse().ok()?;
    let h = it.next()?.parse().ok()?;
    Some((w, h))
}

pub fn parse_board<R: BufRead>(reader: &mut R, first: &str) -> Option<Board> {
    let (width, height) = parse_dims(first)?;
    read_line(reader)?;
    let mut cells = Vec::with_capacity(height);
    for _ in 0..height {
        let line = read_line(reader)?;
        let body: Vec<char> = line.chars().skip(4).take(width).collect();
        if body.len() < width {
            return None;
        }
        cells.push(body);
    }
    Some(Board { width, height, cells })
}

pub fn parse_piece<R: BufRead>(reader: &mut R, first: &str) -> Option<Piece> {
    let (width, height) = parse_dims(first)?;
    let mut cells = Vec::new();
    for y in 0..height {
        let line = read_line(reader)?;
        for (x, ch) in line.chars().take(width).enumerate() {
            if ch != '.' && !ch.is_whitespace() {
                cells.push((x, y));
            }
        }
    }
    Some(Piece { width, height, cells })
}

pub fn is_valid(board: &Board, piece: &Piece, x: usize, y: usize, player: u8) -> bool {
    let (mine, theirs) = symbols(player);
    let mut overlap = 0;
    for &(dx, dy) in &piece.cells {
        let (cx, cy) = (x + dx, y + dy);
        if cx >= board.width || cy >= board.height {
            return false;
        }
        let c = board.cells[cy][cx];
        if theirs.contains(&c) {
            return false;
        }
        if mine.contains(&c) {
            overlap += 1;
            if overlap > 1 {
                return false;
            }
        }
    }
    overlap == 1
}

fn distance_map(board: &Board, player: u8) -> Vec<Vec<i32>> {
    let (_, theirs) = symbols(player);
    let mut dist = vec![vec![-1; board.width]; board.height];
    let mut q = VecDeque::new();
    for y in 0..board.height {
        for x in 0..board.width {
            if theirs.contains(&board.cells[y][x]) {
                dist[y][x] = 0;
                q.push_back((x, y));
            }
        }
    }
    while let Some((x, y)) = q.pop_front() {
        let d = dist[y][x];
        let dirs: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (ddx, ddy) in dirs {
            let nx = x as i32 + ddx;
            let ny = y as i32 + ddy;
            if nx < 0 || ny < 0 || nx >= board.width as i32 || ny >= board.height as i32 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if dist[ny][nx] == -1 {
                dist[ny][nx] = d + 1;
                q.push_back((nx, ny));
            }
        }
    }
    dist
}

pub fn find_best_move(board: &Board, piece: &Piece, player: u8) -> Option<(usize, usize)> {
    if piece.cells.is_empty() {
        return None;
    }
    let dist = distance_map(board, player);
    let no_enemy = dist.iter().all(|r| r.iter().all(|&d| d == -1));
    let (mine, _) = symbols(player);

    let max_x = board.width.saturating_sub(1);
    let max_y = board.height.saturating_sub(1);
    let mut best: Option<((i32, i32), (usize, usize))> = None;

    for y in 0..=max_y {
        for x in 0..=max_x {
            if !is_valid(board, piece, x, y, player) {
                continue;
            }
            let mut sum = 0;
            let mut min_d = i32::MAX;
            for &(dx, dy) in &piece.cells {
                let (cx, cy) = (x + dx, y + dy);
                if mine.contains(&board.cells[cy][cx]) {
                    continue;
                }
                let d = if no_enemy { 0 } else { dist[cy][cx] };
                sum += d;
                min_d = min_d.min(d);
            }
            let score = (min_d, sum);
            if best.map_or(true, |(s, _)| score < s) {
                best = Some((score, (x, y)));
            }
        }
    }
    best.map(|(_, pos)| pos)
}

pub fn format_move(mv: Option<(usize, usize)>) -> String {
    match mv {
        Some((x, y)) => format!("{} {}\n", x, y),
        None => "0 0\n".to_string(),
    }
}

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut player: u8 = 1;
    let mut board: Option<Board> = None;

    while let Some(line) = read_line(&mut reader) {
        let t = line.trim();
        if t.starts_with("$$$") {
            if t.contains("p2") {
                player = 2;
            } else if t.contains("p1") {
                player = 1;
            }
        } else if t.starts_with("Anfield") {
            board = parse_board(&mut reader, t);
        } else if t.starts_with("Piece") {
            let piece = parse_piece(&mut reader, t);
            let mv = match (&board, &piece) {
                (Some(b), Some(p)) => find_best_move(b, p, player),
                _ => None,
            };
            let _ = out.write_all(format_move(mv).as_bytes());
            let _ = out.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const INPUT: &str = "Anfield 20 15:\n    01234567890123456789\n\
000 ....................\n001 ....................\n002 .........@..........\n\
003 ....................\n004 ....................\n005 ....................\n\
006 ....................\n007 ....................\n008 ....................\n\
009 ....................\n010 ....................\n011 ....................\n\
012 .........$..........\n013 ....................\n014 ....................\n\
Piece 4 1:\n.OO.\n";

    fn load() -> (Board, Piece) {
        let mut r = Cursor::new(INPUT);
        let first = read_line(&mut r).unwrap();
        let b = parse_board(&mut r, &first).unwrap();
        let first = read_line(&mut r).unwrap();
        let p = parse_piece(&mut r, &first).unwrap();
        (b, p)
    }

    #[test]
    fn parses_board_and_piece() {
        let (b, p) = load();
        assert_eq!((b.width, b.height), (20, 15));
        assert_eq!(b.cells[2][9], '@');
        assert_eq!(b.cells[12][9], '$');
        assert_eq!((p.width, p.height), (4, 1));
        assert_eq!(p.cells, vec![(1, 0), (2, 0)]);
    }

    #[test]
    fn valid_placement_needs_exactly_one_own_cell() {
        let (b, p) = load();
        assert!(is_valid(&b, &p, 7, 2, 1));
        assert!(is_valid(&b, &p, 8, 2, 1));
        assert!(!is_valid(&b, &p, 9, 2, 1));
    }

    #[test]
    fn rejects_zero_overlap_and_double_overlap() {
        let (b, p) = load();
        assert!(!is_valid(&b, &p, 0, 0, 1));
        let mut b2 = b.clone();
        b2.cells[2][8] = '@';
        assert!(!is_valid(&b2, &p, 7, 2, 1));
    }

    #[test]
    fn rejects_overlap_with_opponent() {
        let (mut b, p) = load();
        b.cells[2][8] = 's';
        assert!(!is_valid(&b, &p, 7, 2, 1));
    }

    #[test]
    fn last_placed_lowercase_counts_as_own() {
        let (mut b, p) = load();
        b.cells[2][9] = 'a';
        assert!(is_valid(&b, &p, 7, 2, 1));
    }

    #[test]
    fn never_goes_out_of_bounds() {
        let (b, p) = load();
        assert!(!is_valid(&b, &p, 18, 2, 1));
        assert!(!is_valid(&b, &p, 0, 15, 1));
        let mv = find_best_move(&b, &p, 1).unwrap();
        assert!(is_valid(&b, &p, mv.0, mv.1, 1));
    }

    #[test]
    fn output_format() {
        assert_eq!(format_move(Some((7, 2))), "7 2\n");
        assert_eq!(format_move(None), "0 0\n");
    }

    #[test]
    fn plays_two_turns_in_a_row() {
        let two = format!("{}{}", INPUT, INPUT.replace("$$$", "$$$"));
        let mut r = Cursor::new(two);
        let mut answers = 0;
        let mut board = None;
        while let Some(line) = read_line(&mut r) {
            let t = line.trim().to_string();
            if t.starts_with("Anfield") {
                board = parse_board(&mut r, &t);
            } else if t.starts_with("Piece") {
                let p = parse_piece(&mut r, &t).unwrap();
                assert!(find_best_move(board.as_ref().unwrap(), &p, 1).is_some());
                answers += 1;
            }
        }
        assert_eq!(answers, 2);
    }
}