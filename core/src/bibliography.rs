//! The scientific references behind the app's rules, as an academic bibliography: who wrote
//! what, where, and a public page for it (a PubMed record, a DOI landing page, a WHO or DTIC page).
//! Reading a study in full may cost money; the reference itself never should.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    BodyComposition,
    HealthyRanges,
    Energy,
    Activity,
    Macronutrients,
    Water,
}

pub const TOPICS: [Topic; 6] = [
    Topic::BodyComposition,
    Topic::HealthyRanges,
    Topic::Energy,
    Topic::Activity,
    Topic::Macronutrients,
    Topic::Water,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference {
    pub topic: Topic,
    pub authors: &'static str,
    pub year: u16,
    pub title: &'static str,
    pub published_in: &'static str,
    pub page: Option<&'static str>,
    pub used_for: &'static str,
}

pub fn references_about(topic: Topic) -> Vec<Reference> {
    REFERENCES
        .iter()
        .filter(|reference| reference.topic == topic)
        .copied()
        .collect()
}

pub const REFERENCES: [Reference; 27] = [
    Reference {
        topic: Topic::BodyComposition,
        authors: "Hodgdon J. A., Beckett M. B.",
        year: 1984,
        title: "Prediction of percent body fat for U.S. Navy men from body circumferences and height",
        published_in: "Naval Health Research Center, San Diego, report 84-11",
        page: Some("https://apps.dtic.mil/sti/pdfs/ADA143890.pdf"),
        used_for: "A man's fat ratio, read from his waist, neck and height",
    },
    Reference {
        topic: Topic::BodyComposition,
        authors: "Hodgdon J. A., Beckett M. B.",
        year: 1984,
        title: "Prediction of percent body fat for U.S. Navy women from body circumferences and height",
        published_in: "Naval Health Research Center, San Diego, report 84-29",
        page: Some("https://apps.dtic.mil/sti/html/tr/ADA146456/index.html"),
        used_for: "A woman's fat ratio, read from her waist, hip, neck and height",
    },
    Reference {
        topic: Topic::HealthyRanges,
        authors: "World Health Organization",
        year: 2000,
        title: "Obesity: preventing and managing the global epidemic. Report of a WHO consultation",
        published_in: "WHO Technical Report Series 894",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/11234459/"),
        used_for: "The healthy body mass index, 18.5 to 24.9, behind the recommended mass",
    },
    Reference {
        topic: Topic::HealthyRanges,
        authors: "Gallagher D., Heymsfield S. B., Heo M., Jebb S. A., Murgatroyd P. R., Sakamoto Y.",
        year: 2000,
        title: "Healthy percentage body fat ranges: an approach for developing guidelines based on body mass index",
        published_in: "American Journal of Clinical Nutrition 72(3):694-701",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/10966886/"),
        used_for: "The healthy body fat by age and gender (table 4), behind the recommended fat",
    },
    Reference {
        topic: Topic::Energy,
        authors: "McArdle W. D., Katch F. I., Katch V. L.",
        year: 2015,
        title: "Exercise physiology: nutrition, energy, and human performance (8th edition)",
        published_in: "Wolters Kluwer Health",
        page: None,
        used_for: "The basal rate from lean mass (Katch-McArdle)",
    },
    Reference {
        topic: Topic::Energy,
        authors: "Elia M.",
        year: 1992,
        title: "Organ and tissue contribution to metabolic rate",
        published_in: "In Kinney J. M., Tucker H. N. (eds), Energy metabolism: tissue determinants and cellular corollaries, Raven Press, 61-79",
        page: None,
        used_for: "What the organs cost, per kilo of lean and fat mass",
    },
    Reference {
        topic: Topic::Energy,
        authors: "Calcagno M., Kahleova H., Alwarith J., Burgess N. N., Flores R. A., Busta M. L., Barnard N. D.",
        year: 2019,
        title: "The thermic effect of food: a review",
        published_in: "Journal of the American College of Nutrition 38(6):547-551",
        page: Some("https://doi.org/10.1080/07315724.2018.1552544"),
        used_for: "Digestion priced on the grams of protein, carbohydrate, fat and fibre eaten",
    },
    Reference {
        topic: Topic::Energy,
        authors: "Levine J. A.",
        year: 2002,
        title: "Non-exercise activity thermogenesis (NEAT)",
        published_in: "Best Practice & Research Clinical Endocrinology & Metabolism 16(4):679-702",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/12468415/"),
        used_for: "The day off the clock: sitting, standing, moving and fidgeting",
    },
    Reference {
        topic: Topic::Energy,
        authors: "Alpert S. S.",
        year: 2005,
        title: "A limit on the energy transfer rate from the human fat store in hypophagia",
        published_in: "Journal of Theoretical Biology 233(1):1-13",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/15615615/"),
        used_for: "The band's floor: what the fat store can spare in a day, about 48.5 kcal per kilo",
    },
    Reference {
        topic: Topic::Energy,
        authors: "Loucks A. B., Kiens B., Wright H. H.",
        year: 2011,
        title: "Energy availability in athletes",
        published_in: "Journal of Sports Sciences 29(sup1):S7-S15",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/21793767/"),
        used_for: "The band's minimum per kilo of lean mass, plus the burn",
    },
    Reference {
        topic: Topic::Activity,
        authors: "Herrmann S. D., Willis E. A., Ainsworth B. E., et al.",
        year: 2024,
        title: "2024 Adult Compendium of Physical Activities: a third update of the energy costs of human activities",
        published_in: "Journal of Sport and Health Science 13(1):6-12",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/38242596/"),
        used_for: "The MET of each exercise, burnt above rest",
    },
    Reference {
        topic: Topic::Activity,
        authors: "Paluch A. E., Bajpai S., Bassett D. R., et al.",
        year: 2022,
        title: "Daily steps and all-cause mortality: a meta-analysis of 15 international cohorts",
        published_in: "The Lancet Public Health 7(3):e219-e228",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/35247352/"),
        used_for: "The usual day's steps: 8000 a day under 60 and 6000 from 60, past which the risk of dying stops falling",
    },
    Reference {
        topic: Topic::Activity,
        authors: "Buckley J. P., Hedge A., Yates T., et al.",
        year: 2015,
        title: "The sedentary office: an expert statement on the growing case for change towards better health and productivity",
        published_in: "British Journal of Sports Medicine 49(21):1357-1362",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/26034192/"),
        used_for: "The usual day's 2 hours standing, the first step towards 4",
    },
    Reference {
        topic: Topic::Activity,
        authors: "Ekelund U., Steene-Johannessen J., Brown W. J., et al.",
        year: 2016,
        title: "Does physical activity attenuate, or even eliminate, the detrimental association of sitting time with mortality? A harmonised meta-analysis of data from more than 1 million men and women",
        published_in: "The Lancet 388(10051):1302-1310",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/27475271/"),
        used_for: "The usual day's 8 hours sitting at most, past which the least active die sooner",
    },
    Reference {
        topic: Topic::Activity,
        authors: "Bull F. C., Al-Ansari S. S., Biddle S., et al.",
        year: 2020,
        title: "World Health Organization 2020 guidelines on physical activity and sedentary behaviour",
        published_in: "British Journal of Sports Medicine 54(24):1451-1462",
        page: Some("https://doi.org/10.1136/bjsports-2020-102955"),
        used_for: "Less sitting and more moving, at any intensity: the direction of the usual day",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "Jäger R., Kerksick C. M., Campbell B. I., et al.",
        year: 2017,
        title: "International Society of Sports Nutrition position stand: protein and exercise",
        published_in: "Journal of the International Society of Sports Nutrition 14:20",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/28642676/"),
        used_for: "Protein of 1.4 to 2.0 g per kilo for those who train",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "Morton R. W., Murphy K. T., McKellar S. R., et al.",
        year: 2018,
        title: "A systematic review, meta-analysis and meta-regression of the effect of protein supplementation on resistance training-induced gains in muscle mass and strength in healthy adults",
        published_in: "British Journal of Sports Medicine 52(6):376-384",
        page: Some("https://pubmed.ncbi.nlm.nih.gov/28698222/"),
        used_for: "Protein of 1.6 g per kilo, where the gains in lean mass from training level off",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
        year: 2010,
        title: "Scientific Opinion on Dietary Reference Values for fats, including saturated fatty acids, polyunsaturated fatty acids, monounsaturated fatty acids, trans fatty acids, and cholesterol",
        published_in: "EFSA Journal 8(3):1461",
        page: Some("https://doi.org/10.2903/j.efsa.2010.1461"),
        used_for: "Fat of 20 to 35% of the day's energy",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "World Health Organization",
        year: 2023,
        title: "Total fat intake for the prevention of unhealthy weight gain in adults and children: WHO guideline",
        published_in: "World Health Organization, Geneva",
        page: Some("https://www.who.int/publications/i/item/9789240073654"),
        used_for: "Fat of 30% of the day's energy at most",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
        year: 2010,
        title: "Scientific Opinion on Dietary Reference Values for carbohydrates and dietary fibre",
        published_in: "EFSA Journal 8(3):1462",
        page: Some("https://doi.org/10.2903/j.efsa.2010.1462"),
        used_for: "Fibre of 25 g a day for adults",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "World Health Organization",
        year: 2023,
        title: "Carbohydrate intake for adults and children: WHO guideline",
        published_in: "World Health Organization, Geneva",
        page: Some("https://www.who.int/publications/i/item/9789240073593"),
        used_for: "Fibre of at least 25 g a day, behind the recommended fibre",
    },
    Reference {
        topic: Topic::Macronutrients,
        authors: "Institute of Medicine",
        year: 2005,
        title: "Dietary Reference Intakes for energy, carbohydrate, fiber, fat, fatty acids, cholesterol, protein, and amino acids",
        published_in: "The National Academies Press, Washington",
        page: Some("https://doi.org/10.17226/10490"),
        used_for: "At least 130 g of carbohydrates, and fibre of 14 g per 1000 kcal",
    },
    Reference {
        topic: Topic::Water,
        authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
        year: 2010,
        title: "Scientific Opinion on Dietary Reference Values for water",
        published_in: "EFSA Journal 8(3):1459",
        page: Some("https://doi.org/10.2903/j.efsa.2010.1459"),
        used_for: "Before a lean mass is known, water of 2.5 L a day for men and 2.0 L for women, four fifths of it from drinks",
    },
    Reference {
        topic: Topic::Water,
        authors: "Wang Z., Deurenberg P., Wang W., et al.",
        year: 1999,
        title: "Hydration of fat-free body mass: new physiological modeling approach",
        published_in: "American Journal of Physiology, Endocrinology and Metabolism 276(6):E995-E1003",
        page: Some("https://doi.org/10.1152/ajpendo.1999.276.6.E995"),
        used_for: "Lean mass holding 73.2% water, the water a body turns over being drawn from it",
    },
    Reference {
        topic: Topic::Water,
        authors: "Yamada Y., Zhang X., Henderson M. E. T., et al.",
        year: 2022,
        title: "Variation in human water turnover associated with environmental and lifestyle factors",
        published_in: "Science 378(6622):909-915",
        page: Some("https://doi.org/10.1126/science.abm8668"),
        used_for: "The day's target: about 7% of the body's water turned over a day, nine tenths of it from drinks",
    },
    Reference {
        topic: Topic::Water,
        authors: "Westerterp K. R., Plasqui G., Goris A. H. C.",
        year: 2005,
        title: "Water loss as a function of energy intake, physical activity and season",
        published_in: "British Journal of Nutrition 93(2):199-203",
        page: Some(
            "https://www.cambridge.org/core/journals/british-journal-of-nutrition/article/water-loss-as-a-function-of-energy-intake-physical-activity-and-season/51D08EEA77F1019DD393667429046510",
        ),
        used_for: "The floor of a millilitre per kcal spent, and 1.25 mL per kcal burnt in the target",
    },
    Reference {
        topic: Topic::Water,
        authors: "Joo M. A., Kim E. Y.",
        year: 2013,
        title: "Hyponatremia caused by excessive intake of water as a form of child abuse",
        published_in: "Annals of Pediatric Endocrinology & Metabolism 18(2):95-98",
        page: Some("https://doi.org/10.6065/apem.2013.18.2.95"),
        used_for: "Kidneys clearing no more than 0.8 to 1 L of water an hour, the reason to spread the cups out",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_topic_gathers_its_own_references_and_every_reference_falls_under_one() {
        let water = references_about(Topic::Water);
        let gathered: usize = TOPICS
            .iter()
            .map(|topic| references_about(*topic).len())
            .sum();

        assert_eq!(water.len(), 5);
        assert!(
            water
                .iter()
                .all(|reference| reference.topic == Topic::Water)
        );
        assert_eq!(gathered, REFERENCES.len());
    }

    #[test]
    fn every_public_page_is_a_secure_link() {
        assert!(
            REFERENCES
                .iter()
                .filter_map(|reference| reference.page)
                .all(|page| page.starts_with("https://"))
        );
    }

    #[test]
    fn the_sources_of_the_recommended_protein_and_usual_day_are_cited() {
        let cited = |topic: Topic| -> Vec<(&str, u16)> {
            references_about(topic)
                .iter()
                .map(|reference| (reference.authors, reference.year))
                .collect()
        };

        assert!(
            cited(Topic::Macronutrients)
                .contains(&("Morton R. W., Murphy K. T., McKellar S. R., et al.", 2018))
        );
        for source in [
            (
                "Ekelund U., Steene-Johannessen J., Brown W. J., et al.",
                2016,
            ),
            ("Buckley J. P., Hedge A., Yates T., et al.", 2015),
            ("Paluch A. E., Bajpai S., Bassett D. R., et al.", 2022),
        ] {
            assert!(cited(Topic::Activity).contains(&source), "{source:?}");
        }
    }

    #[test]
    fn the_sources_of_the_hydrate_band_and_its_cap_are_cited() {
        let cited: Vec<(&str, u16)> = references_about(Topic::Water)
            .iter()
            .map(|reference| (reference.authors, reference.year))
            .collect();

        for source in [
            ("Wang Z., Deurenberg P., Wang W., et al.", 1999),
            ("Yamada Y., Zhang X., Henderson M. E. T., et al.", 2022),
            ("Westerterp K. R., Plasqui G., Goris A. H. C.", 2005),
            ("Joo M. A., Kim E. Y.", 2013),
        ] {
            assert!(cited.contains(&source), "{source:?}");
        }
    }
}
