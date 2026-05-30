use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    #[serde(rename = "EMERALD")]
    Emerald,
    #[serde(rename = "ORE")]
    Ore,
    #[serde(rename = "WOOD")]
    Wood,
    #[serde(rename = "FISH")]
    Fish,
    #[serde(rename = "CROP")]
    Crop,
}

/// Base resource generation values
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub struct BaseResGen {
    pub emerald: i32,
    pub ore: i32,
    pub crop: i32,
    pub fish: i32,
    pub wood: i32,
}

impl BaseResGen {
    pub const fn has_emerald(&self) -> bool {
        self.emerald > 9000
    }

    pub const fn has_ore(&self) -> bool {
        self.ore != 0
    }

    pub const fn has_crop(&self) -> bool {
        self.crop != 0
    }

    pub const fn has_fish(&self) -> bool {
        self.fish != 0
    }

    pub const fn has_wood(&self) -> bool {
        self.wood != 0
    }

    pub const fn has_double_ore(&self) -> bool {
        self.ore >= 7200
    }

    pub const fn has_double_crop(&self) -> bool {
        self.crop >= 7200
    }

    pub const fn has_double_fish(&self) -> bool {
        self.fish >= 7200
    }

    pub const fn has_double_wood(&self) -> bool {
        self.wood >= 7200
    }

    pub const fn has_res(&self) -> (bool, bool, bool, bool, bool) {
        (
            self.has_emerald(),
            self.has_crop(),
            self.has_fish(),
            self.has_ore(),
            self.has_wood(),
        )
    }

    pub const fn has_double_res(&self) -> (bool, bool, bool, bool) {
        (
            self.has_double_crop(),
            self.has_double_fish(),
            self.has_double_ore(),
            self.has_double_wood(),
        )
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Default)]
pub struct Resources {
    pub emerald: ResourceValues,
    pub ore: ResourceValues,
    pub crop: ResourceValues,
    pub fish: ResourceValues,
    pub wood: ResourceValues,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Default)]
pub struct ResourceValues {
    pub generation: i32,
    pub stored: i32,
    pub limit: i32,
}

impl From<ResourceValues> for (i32, i32, i32) {
    fn from(value: ResourceValues) -> Self {
        (value.generation, value.stored, value.limit)
    }
}

impl From<(i32, i32, i32)> for ResourceValues {
    fn from((g, s, l): (i32, i32, i32)) -> Self {
        Self {
            generation: g,
            stored: s,
            limit: l,
        }
    }
}
