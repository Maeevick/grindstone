use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub authors: &'static str,
    pub year: u16,
    pub title: &'static str,
    pub link: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    pub about: &'static str,
    pub says: &'static str,
    pub source: Source,
}

const IOM_2005: Source = Source {
    authors: "Institute of Medicine",
    year: 2005,
    title: "Dietary Reference Intakes for Energy, Carbohydrate, Fiber, Fat, Fatty Acids, Cholesterol, Protein, and Amino Acids",
    link: "https://doi.org/10.17226/10490",
};

pub const FUEL_RECOMMENDATIONS: [Recommendation; 6] = [
    Recommendation {
        about: "Protein",
        says: "1.4 to 2.0 g per kg of body mass a day for most people who exercise",
        source: Source {
            authors: "Jäger R. et al., International Society of Sports Nutrition",
            year: 2017,
            title: "Position stand: protein and exercise",
            link: "https://doi.org/10.1186/s12970-017-0177-8",
        },
    },
    Recommendation {
        about: "Protein",
        says: "0.83 g per kg of body mass a day covers almost every adult (population reference intake)",
        source: Source {
            authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
            year: 2012,
            title: "Scientific Opinion on Dietary Reference Values for protein",
            link: "https://doi.org/10.2903/j.efsa.2012.2557",
        },
    },
    Recommendation {
        about: "Fat",
        says: "20 to 35% of the day's energy",
        source: Source {
            authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
            year: 2010,
            title: "Scientific Opinion on Dietary Reference Values for fats",
            link: "https://doi.org/10.2903/j.efsa.2010.1461",
        },
    },
    Recommendation {
        about: "Fibre",
        says: "25 g a day for adults (adequate intake)",
        source: Source {
            authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
            year: 2010,
            title: "Scientific Opinion on Dietary Reference Values for carbohydrates and dietary fibre",
            link: "https://doi.org/10.2903/j.efsa.2010.1462",
        },
    },
    Recommendation {
        about: "Fibre and carbohydrates",
        says: "14 g of fibre per 1000 kcal, and at least 130 g of carbohydrates a day",
        source: IOM_2005,
    },
    Recommendation {
        about: "Energy floor",
        says: "A fat store releases at most about 22 kcal per pound (48.5 kcal per kg) of fat a day",
        source: Source {
            authors: "Alpert S. S.",
            year: 2005,
            title: "A limit on the energy transfer rate from the human fat store in hypophagia",
            link: "https://doi.org/10.1016/j.jtbi.2004.08.029",
        },
    },
];
