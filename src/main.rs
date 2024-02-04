use std::fmt::format;
use std::io::stdout;
use std::io::Write;
use std::{cmp , env , fs , io};
use std::cmp::Ordering; 
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent , KeyModifiers};
use crossterm::{event, terminal , execute , cursor , queue , style};

use crossterm::terminal::ClearType;

fn main() -> crossterm::Result<()> {

    let _cleanup = CleanUp;

    let _stdio = terminal::enable_raw_mode()?;

    let mut editor = Editor::new();

    while editor.run()?{}
    Ok(())
    
}

struct CleanUp;

impl Drop for CleanUp{
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        Output::clear_screen().expect("Error");
    }
}

struct Reader;

impl Reader{
    fn read_key(&self) -> crossterm::Result<KeyEvent>{
        loop{
            if event::poll(Duration::from_millis(500))?{
                if let Event::Key(event) = event::read() ? {
                    return Ok(event);
                }
            }
        }
    }
}

struct Output{
    win_size:(usize , usize),
    editor_contents: EditorContents,
    cursor:CursorController,
    editor_rows:EditorRows,
    status_message:StatusMessage,
    dirty:u64
}

impl Output{
    fn new() -> Self{
        let win_size = terminal::size()
        .map(|(x,y)| (x as usize , y as usize - 2))
        .unwrap();

        Self {
            win_size,
            editor_contents:EditorContents::new(),
            cursor:CursorController::new(win_size),
            editor_rows:EditorRows::new(),
            status_message:StatusMessage::new("HELP: Ctrl-S = Save | Ctrl-Q = Quit ".into()),
            dirty:0
        }
    }

    fn insert_newline(&mut self){
        if self.cursor.cursor_x == 0{
            self.editor_rows.insert_row(self.cursor.cursor_y, String::new());
        }else {
            let cur_row = self.editor_rows.get_editor_row_mut(self.cursor.cursor_y);
            let new_row_content:String = cur_row.row_contents[self.cursor.cursor_x..].into();

            cur_row.row_contents.truncate(self.cursor.cursor_x);
            EditorRows::render_row(cur_row);

            self.editor_rows.insert_row(self.cursor.cursor_y+1, new_row_content)
        }
        self.cursor.cursor_x = 0;
        self.cursor.cursor_y += 1;
        self.dirty += 1;
    }

    fn insert_char(&mut self , ch:char){
        if self.cursor.cursor_y == self.editor_rows.num_rows(){
            self.editor_rows.insert_row(self.editor_rows.num_rows(), String::new());
            self.dirty += 1;
        }
        self.editor_rows.get_editor_row_mut(self.cursor.cursor_y).insert_char(self.cursor.cursor_x, ch);

        self.cursor.cursor_x += 1;
        self.dirty += 1;
    }

    fn delete_char(&mut self){
        if self.cursor.cursor_y == self.editor_rows.num_rows(){
            return;
        }
        let row = self.editor_rows.get_editor_row_mut(self.cursor.cursor_y);

        if self.cursor.cursor_x > 0{
            row.delete_char(self.cursor.cursor_x - 1);
            self.cursor.cursor_x -= 1;
        }else{
            let prev_row_content = self.editor_rows.get_row(self.cursor.cursor_y - 1);
            self.cursor.cursor_x = prev_row_content.len();
            self.editor_rows.join_adjacent_rows(self.cursor.cursor_y);
            self.cursor.cursor_y -= 1;
        }
        self.dirty += 1;


    }

    fn clear_screen() -> crossterm::Result<()>{
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout() , cursor::MoveTo(0,0))
    }

    fn draw_status_bar(&mut self) {
        self.editor_contents
            .push_str(&style::Attribute::Reverse.to_string());
        let info = format!(
            "{} {} -- {} lines",
            self.editor_rows
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"), if self.dirty > 0{"(modified)"} else{""},
            self.editor_rows.num_rows()
        );
        let info_len = cmp::min(info.len(), self.win_size.0);
        let line_info = format!(
            "{}/{}",
            self.cursor.cursor_y + 1,
            self.editor_rows.num_rows()
        );
        self.editor_contents.push_str(&info[..info_len]);
        for i in info_len..self.win_size.0 {
            if self.win_size.0 - i == line_info.len() {
                self.editor_contents.push_str(&line_info);
                break;
            } else {
                self.editor_contents.push(' ')
            }
        }
        self.editor_contents
            .push_str(&style::Attribute::Reset.to_string());
        self.editor_contents.push_str("\r\n");
    }

    fn draw_message_bar(&mut self) {
        queue!(
            self.editor_contents,
            terminal::Clear(ClearType::UntilNewLine)
        ).unwrap();
        
        if let Some(msg) = self.status_message.message(){
            self.editor_contents.push_str(&msg[..cmp::min(self.win_size.0, msg.len())]);
        }
    }

    fn add_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            let file_row = i + self.cursor.row_offset;
            if file_row >= self.editor_rows.num_rows() {
                if self.editor_rows.num_rows() == 0 && i == screen_rows / 3 {
                    let mut welcome = String::from("Rust Editor --- Version 1");
                    if welcome.len() > screen_columns {
                        welcome.truncate(screen_columns)
                    }
                    let mut padding = (screen_columns - welcome.len()) / 2;
                    if padding != 0 {
                        self.editor_contents.push('~');
                        padding -= 1
                    }
                    (0..padding).for_each(|_| self.editor_contents.push(' '));
                    self.editor_contents.push_str(&welcome);
                } else {
                    self.editor_contents.push('~');
                }
            } else {
                let row = self.editor_rows.get_render(file_row);
                let col_offset = self.cursor.col_offset;

                let len = cmp::min(row.len().saturating_sub(col_offset) , screen_columns);
                let start = if len == 0{0} else{col_offset};
                self.editor_contents.push_str(&row[start..start + len]);
            }
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();
            self.editor_contents.push_str("\r\n");
            
        }
    }

    fn refresh_screen(&mut self) ->crossterm::Result<()>{
        self.cursor.scroll(&self.editor_rows);
        queue!(self.editor_contents, cursor::Hide ,cursor::MoveTo(0,0))?;
        self.add_rows();
        self.draw_status_bar();
        self.draw_message_bar();
        
        let cursor_x = self.cursor.render_x - self.cursor.col_offset;

        let cursor_y = self.cursor.cursor_y - self.cursor.row_offset ;
        
        queue!(self.editor_contents, cursor::MoveTo(cursor_x  as u16,cursor_y  as u16) ,cursor::Show)?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self,direction:KeyCode) {
        self.cursor.move_cursor(direction , &self.editor_rows);
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    screen_columns:usize,
    screen_rows:usize,
    row_offset:usize,
    col_offset:usize,
    render_x: usize,
    
}

impl CursorController{
    fn new(win_size:(usize , usize)) -> Self{
        Self{
            cursor_x:0,
            cursor_y:0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
            row_offset:0,
            col_offset:0,
            render_x:0
        }
    }

    fn scroll(&mut self, editor_rows: &EditorRows){
        self.render_x = 0;
        if self.cursor_y < editor_rows.num_rows(){
            self.render_x = self.get_render_x(editor_rows.get_editor_row(self.cursor_y))
        }

        self.row_offset = cmp::min(self.row_offset , self.cursor_y);

        if self.cursor_y >= self.row_offset + self.screen_rows{
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }

        self.col_offset = cmp::min(self.col_offset, self.render_x);

        if self.render_x >= self.col_offset + self.screen_columns{
            self.col_offset = self.render_x - self.screen_columns + 1;
        }

    }

    fn move_cursor(&mut self, direction: KeyCode , editor_rows: &EditorRows) {
        let num_rows = editor_rows.num_rows();
        match direction {
            KeyCode::Up => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            },
            KeyCode::Left => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            },
            KeyCode::Down => {
                if self.cursor_y < num_rows{
                    self.cursor_y += 1;
                }
            },
            KeyCode::Right => {
                
                if self.cursor_y < num_rows{
                    match self.cursor_x.cmp(&editor_rows.get_row(self.cursor_y).len() ){
                        Ordering::Less => self.cursor_x += 1,
                        Ordering::Equal => {
                            self.cursor_y += 1;
                            return self.cursor_x = 0;
                        }
                        _ => unimplemented!(),
                    }
                }
                
            },
            KeyCode::Home => {
                self.cursor_x = 0;
            },
            KeyCode::End => {
                if self.cursor_y < num_rows{
                    self.cursor_x = editor_rows.get_row(self.cursor_y).len();
                }
            },
            _ => unimplemented!(),
        }
        let row_len = if self.cursor_y < num_rows{
            editor_rows.get_row(self.cursor_y).len()
        }else{
            0
        };
        self.cursor_x = cmp::min(self.cursor_x, row_len);
    }

    fn get_render_x(&self, row: &Row) -> usize {
        row.row_contents[..self.cursor_x]
            .chars()
            .fold(0, |render_x, c| {
                if c == '\t' {
                    render_x + TAB_STOP - (render_x % TAB_STOP)
                } else {
                    render_x + 1
                }
            })
    }
}

struct EditorContents{
    content:String
}

impl EditorContents{
    fn new() -> Self{
        Self{
            content:String::new()
        }
    }

    fn push(&mut self , ch:char){
        self.content.push(ch);
    }
    
    fn push_str(&mut self , st:&str){
        self.content.push_str(st);
    }
}

impl std::io::Write for EditorContents{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match std::str::from_utf8(buf){
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(std::io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let out = write!(stdout(), "{}" , self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct StatusMessage {
    message: Option<String>,
    set_time: Option<Instant>,
}

impl StatusMessage {
    fn new(initial_message: String) -> Self {
        Self {
            message: Some(initial_message),
            set_time: Some(Instant::now()),
        }
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
        self.set_time = Some(Instant::now())
    }

    fn message(&mut self) -> Option<&String> {
        self.set_time.and_then(|time| {
            if time.elapsed() > Duration::from_secs(5) {
                self.message = None;
                self.set_time = None;
                None
            } else {
                Some(self.message.as_ref().unwrap())
            }
        })
    }
}

#[derive(Default)]
struct Row{
    row_contents:String,
    render:String
}

impl Row{
    fn new(row_contents: String, render: String) -> Self{
        Self { row_contents, render }
    }

    fn insert_char(&mut self , at:usize , ch:char){
        self.row_contents.insert(at, ch);
        EditorRows::render_row(self);
    }

    fn delete_char(&mut self , at:usize){
        self.row_contents.remove(at);
        EditorRows::render_row(self)
    }
}

const TAB_STOP: usize = 8;
struct EditorRows{
    row_contents:Vec<Row>,
    filename: Option<PathBuf>
}

impl EditorRows{
    fn new() -> Self{
        let mut arg = env::args();

        match arg.nth(1) {
            None => Self{
                row_contents:Vec::new(),
                filename:None
            },
            Some(file) => Self::from_file(file.into())
        }
    }

    fn from_file(file: PathBuf) -> Self{
        let file_contents = fs::read_to_string(&file).expect("Error in reading File");

        Self{
            filename:Some(file),
            row_contents:file_contents.lines().map(|it|{
                let mut row = Row::new(it.into(), String::new());
                Self::render_row(&mut row);
                row
            }).collect(),
        }
    }

    fn insert_row(&mut self, at:usize , contents:String) {
        let mut new_row = Row::new(contents , String::new());
        EditorRows::render_row(&mut new_row);
        self.row_contents.insert(at,new_row);
    }

    fn get_render(&self , at:usize) -> &String{
        &self.row_contents[at].render
    }

    fn get_editor_row(&self , at:usize) -> &Row{
        &self.row_contents[at]
    }

    fn get_editor_row_mut(&mut self , at:usize) -> &mut Row{
        &mut self.row_contents[at]
    }

    fn num_rows(&self) -> usize{
        self.row_contents.len()
    }

    fn render_row(row:&mut Row){
        let mut idx:usize = 0;
        let capacity = row.row_contents.chars().fold(0, |acc, next| acc + if next == '\t' { TAB_STOP } else { 1 });

        row.render = String::with_capacity(capacity);

        row.row_contents.chars().for_each(|c: char| {
            idx += 1;
            if c == '\t'{
                row.render.push(' ');
                while idx % TAB_STOP != 0{
                    row.render.push(' ');
                    idx += 1;
                }
            } else{
                row.render.push(c);
            }
        })
    }

    fn get_row(&self , at:usize) -> &str{
        &self.row_contents[at].row_contents
    }

    fn save(&self) -> io::Result<usize>{
        match &self.filename{
            None => Err(io::Error::new(io::ErrorKind::Other , "no file name specified")),
            Some(name) => {
                let mut file = fs::OpenOptions::new().write(true).create(true).open(name)?;
                let contents: String = self.row_contents
                    .iter()
                    .map(|it| it.row_contents.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n");

                file.set_len(contents.len() as u64);
                file.write_all(contents.as_bytes())?;
                Ok(contents.as_bytes().len())
            }
        }
    }

    fn join_adjacent_rows(&mut self , at:usize){
        let cur_row = self.row_contents.remove(at);
        let prev_row = self.get_editor_row_mut(at - 1);
        prev_row.row_contents.push_str(&cur_row.row_contents);
        Self::render_row(prev_row);
    }
}

const QUIT_TIMES:u8 = 3;

struct Editor{
    reader:Reader,
    output: Output,
    quit_times:u8
}

impl Editor{
    fn new() -> Self{
        Self { 
            reader: Reader, 
            output: Output::new(),
            quit_times: QUIT_TIMES
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool>{
        match self.reader.read_key()?{
            KeyEvent{
                code:KeyCode::Char('q'),
                modifiers:KeyModifiers::CONTROL
            } => {
                if self.output.dirty > 0 && self.quit_times > 0{
                    self.output.status_message.set_message(format!(
                        "WARNING! File has unsaved changes. Press CTRL-Q {} more times to quit",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(true);
                }
                return Ok(false);
            },
            KeyEvent {
                code: direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                                            | KeyCode::Home | KeyCode::End),
                modifiers: KeyModifiers::NONE,
            } => self.output.move_cursor(direction),
            KeyEvent {
                code: val @ (KeyCode::PageUp | KeyCode::PageDown),
                modifiers: KeyModifiers::NONE,
            } => {
                if matches!(val , KeyCode::PageUp){
                    self.output.cursor.cursor_y = self.output.cursor.row_offset
                }else{
                    self.output.cursor.cursor_y = cmp::min(
                        self.output.win_size.1 + self.output.cursor.row_offset - 1,
                        self.output.editor_rows.num_rows()
                    )
                }
            },
            KeyEvent {
                code: code @ (KeyCode::Char(..) | KeyCode::Tab),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => self.output.insert_char(match code {
                KeyCode::Tab => '\t',
                KeyCode::Char(ch) => ch,
                _ => unreachable!(),
            }),
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                if matches!(self.output.editor_rows.filename , None){
                    let prompt = prompt!(&mut self.output , "Save as: {} (ESC to cancel)").map(|it| it.into());
                    if let None = prompt{
                        self.output.status_message.set_message("Save Aborted".into());
                        return Ok(true)
                    }
                    self.output.editor_rows.filename = prompt
                }

                self.output.editor_rows.save().map(|len|{
                    self.output.status_message.set_message(format!("{} bytes written to disk" , len));
                    self.output.dirty = 0
                })?;
                
            },
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => self.output.insert_newline(),
            KeyEvent {
                code: key @ (KeyCode::Backspace | KeyCode::Delete),
                modifiers: KeyModifiers::NONE,
            } => {
                if matches!(key, KeyCode::Delete) {
                    self.output.move_cursor(KeyCode::Right)
                }
                self.output.delete_char()
            }
            _ =>{}
        }
        self.quit_times = QUIT_TIMES;
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

#[macro_export]
macro_rules! prompt {
    ($output:expr,$($args:tt)*) => {{
        let output:&mut Output = &mut $output;
        let mut input = String::with_capacity(32);
        loop {
            output.status_message.set_message(format!($($args)*, input));
            output.refresh_screen()?;
            match Reader.read_key()? {
                KeyEvent {
                    code:KeyCode::Enter,
                    modifiers:KeyModifiers::NONE
                } => {
                    if !input.is_empty() {
                        output.status_message.set_message(String::new());
                        break;
                    }
                },
                KeyEvent {
                    code: KeyCode::Backspace | KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                } =>  {
                    input.pop();
                },
                KeyEvent {
                    code: KeyCode::Esc,
                    ..
                } => {
                    output.status_message.set_message(String::new());
                    input.clear();
                    break;
                },
                KeyEvent {
                    code: code @ (KeyCode::Char(..) | KeyCode::Tab),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                } => input.push(match code {
                        KeyCode::Tab => '\t',
                        KeyCode::Char(ch) => ch,
                        _ => unreachable!(),
                    }),
                _=> {}
            }
        }
        if input.is_empty() { None } else { Some (input) }
    }};
}