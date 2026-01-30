pub struct AsciiArt {
    pub name: &'static str,
    pub small: &'static str,
    pub medium: &'static str,
    pub large: &'static str,
}

pub fn presets() -> Vec<AsciiArt> {
    vec![
        AsciiArt {
            name: "bunny",
            small: r" /\_/\
( •.• )
 /づっ",
            medium: r"  /\_/\
 ( •.• )
 /づっ♡
 /  \",
            large: r"    /\_/\
   ( •.• )
   /づっ♡
  /    \
 (      )
  \__/\_/",
        },
        AsciiArt {
            name: "cat",
            small: r" /\_/\ 
( o.o )
 > ^ <",
            medium: r" /\_/\  
( o.o ) 
 > ^ <  
 /   \",
            large: r" /\_/\      
( o.o )     
 > ^ <      
 /   \     
(     )    
 \___/ ",
        },
        AsciiArt {
            name: "whale",
            small: r"  __
<(o )___
 ( ._> /",
            medium: r"   __
<(o )___
 ( ._> /
  `---'",
            large: r"    __
 <(o )___
  ( ._> /
   `---'
  ~~~~~~~",
        },
    ]
}

pub fn get_ascii(name: &str, size: &str) -> Option<String> {
    let target = name.to_lowercase();
    let size = size.to_lowercase();
    for preset in presets() {
        if preset.name.eq(&target) {
            return Some(match size.as_str() {
                "small" => preset.small.to_string(),
                "large" => preset.large.to_string(),
                _ => preset.medium.to_string(),
            });
        }
    }
    None
}
