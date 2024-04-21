use std::fmt::Display;

#[derive(Copy, Clone)]
pub struct FieldSize {
    width: u16,
    height: u16,
}

impl FieldSize {
    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
    
    pub fn new(w: u16, h: u16) -> FieldSize {
        FieldSize { width: w, height: h }
    }
}

impl Display for FieldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = self.width;
        let h = self.height;

        write!(f, "{w}x{h}")
    }
}

impl TryFrom<&str> for FieldSize {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut w = None;
        let mut h = None;
        for part in value.split('x') {
            if w.is_none() {
                w = match part.parse::<u16>() {
                    Ok(val) => Some(val),
                    Err(e) => return Err(e.to_string()),
                };
            }
            else if h.is_none() {
                h = match part.parse::<u16>() {
                    Ok(val) => Some(val),
                    Err(e) => return Err(e.to_string()),
                };
            }
            else {
                return Err("Expected string of the form <width>x<height> or <length>".into());
            }
        }

        let w = w.unwrap();
        if h.is_none() {
            h = Some(w);
        }
        let h = h.unwrap();

        Ok(FieldSize { width: w, height: h })
    }
}

pub fn parse_field_size(arg: &str) -> Result<FieldSize, String> {
    arg.try_into()
}