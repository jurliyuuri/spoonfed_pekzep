#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Tone {
    No,
    First,
    Second,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Onset {
    P,
    B,
    M,
    C,
    S,
    X,
    Z,
    T,
    D,
    N,
    L,
    K,
    G,
    H,
    Zero,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Vowel {
    A,
    IA,
    UA,
    AI,
    UAI,
    AU,
    IAU,
    UAU,
    E,
    IE,
    UE,
    EI,
    IEI,
    O,
    IO,
    UO,
    I,
    UI,
    U,
    Y,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Coda {
    P,
    T,
    K,
    M,
    N,
    Zero,
}

impl Coda {
    fn to_str(self) -> &'static str {
        match self {
            Coda::P => "p",
            Coda::T => "t",
            Coda::K => "k",
            Coda::M => "m",
            Coda::N => "n",
            Coda::Zero => "",
        }
    }
}

impl Onset {
    fn to_str(self) -> &'static str {
        match self {
            Onset::P => "p",
            Onset::T => "t",
            Onset::K => "k",
            Onset::M => "m",
            Onset::N => "n",
            Onset::Zero => "",
            Onset::B => "b",
            Onset::G => "g",
            Onset::C => "c",
            Onset::S => "s",
            Onset::X => "x",
            Onset::H => "h",
            Onset::Z => "z",
            Onset::L => "l",
            Onset::D => "d",
        }
    }
}

impl Vowel {
    fn to_str(self) -> &'static str {
        match self {
            Vowel::A => "a",
            Vowel::IA => "ia",
            Vowel::UA => "ua",
            Vowel::AI => "ai",
            Vowel::UAI => "uai",
            Vowel::AU => "au",
            Vowel::IAU => "iau",
            Vowel::UAU => "uau",
            Vowel::E => "e",
            Vowel::IE => "ie",
            Vowel::UE => "ue",
            Vowel::EI => "ei",
            Vowel::IEI => "iei",
            Vowel::O => "o",
            Vowel::IO => "io",
            Vowel::UO => "uo",
            Vowel::I => "i",
            Vowel::UI => "ui",
            Vowel::U => "u",
            Vowel::Y => "y",
        }
    }

    fn first_tone_rerrliratixka(self) -> &'static str {
        match self {
            Vowel::A => "ar",
            Vowel::IA => "iar",
            Vowel::UA => "uar",
            Vowel::AI => "ari",
            Vowel::UAI => "uari",
            Vowel::AU => "aru",
            Vowel::IAU => "iaru",
            Vowel::UAU => "uaru",
            Vowel::E => "er",
            Vowel::IE => "ier",
            Vowel::UE => "uer",
            Vowel::EI => "eri",
            Vowel::IEI => "ieri",
            Vowel::O => "or",
            Vowel::IO => "ior",
            Vowel::UO => "uor",
            Vowel::I => "ir",
            Vowel::UI => "uir",
            Vowel::U => "ur",
            Vowel::Y => "yr",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PekZepSyllable {
    onset: Onset,
    vowel: Vowel,
    coda: Coda,
    tone: Tone,
}
impl PekZepSyllable {
    pub fn to_rerrliratixka(self) -> String {
        format!(
            "{}{}{}{}",
            self.onset.to_str(),
            if self.tone == Tone::Second { "r" } else { "" },
            if self.tone == Tone::First {
                self.vowel.first_tone_rerrliratixka()
            } else {
                self.vowel.to_str()
            },
            self.coda.to_str()
        )
    }

    pub fn parse(s: &str) -> Option<PekZepSyllable> {
        let chars: Vec<char> = s.chars().collect();
        let tone = match chars[chars.len() - 1] {
            '1' => Tone::First,
            '2' => Tone::Second,
            _ => Tone::No,
        };

        let onset = match chars[0] {
            'p' => Onset::P,
            'b' => Onset::B,
            'm' => Onset::M,
            'c' => Onset::C,
            's' => Onset::S,
            'x' => Onset::X,
            'z' => Onset::Z,
            't' => Onset::T,
            'd' => Onset::D,
            'n' => Onset::N,
            'l' => Onset::L,
            'k' => Onset::K,
            'g' => Onset::G,
            'h' => Onset::H,
            'a' | 'e' | 'i' | 'u' | 'o' | 'y' => Onset::Zero,
            _ => return None,
        };

        let remaining = chars[(if onset == Onset::Zero { 0 } else { 1 })
            ..chars.len() - (if tone == Tone::No { 0 } else { 1 })]
            .to_vec();

        let coda = match remaining[remaining.len() - 1] {
            'p' => Coda::P,
            't' => Coda::T,
            'k' => Coda::K,
            'm' => Coda::M,
            'n' => Coda::N,
            _ => Coda::Zero,
        };

        let vow = remaining[0..remaining.len() - (if coda == Coda::Zero { 0 } else { 1 })].to_vec();

        let vowel = match vow.as_slice() {
            ['a'] => Vowel::A,
            ['i', 'a'] => Vowel::IA,
            ['u', 'a'] => Vowel::UA,
            ['a', 'i'] => Vowel::AI,
            ['u', 'a', 'i'] => Vowel::UAI,
            ['a', 'u'] => Vowel::AU,
            ['i', 'a', 'u'] => Vowel::IAU,
            ['u', 'a', 'u'] => Vowel::UAU,
            ['e'] => Vowel::E,
            ['i', 'e'] => Vowel::IE,
            ['u', 'e'] => Vowel::UE,
            ['e', 'i'] => Vowel::EI,
            ['i', 'e', 'i'] => Vowel::IEI,
            ['o'] => Vowel::O,
            ['i', 'o'] => Vowel::IO,
            ['u', 'o'] => Vowel::UO,
            ['i'] => Vowel::I,
            ['u', 'i'] => Vowel::UI,
            ['u'] => Vowel::U,
            ['y'] => Vowel::Y,
            _ => return None,
        };

        Some(PekZepSyllable {
            onset,
            vowel,
            coda,
            tone,
        })
    }
}
