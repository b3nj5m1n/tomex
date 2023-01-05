use crate::traits::{CreateByPrompt, Insertable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Genre {
    Fantasy,
    ScienceFiction,
    Dystopian,
    ActionAndAdventure,
    Mystery,
    Horror,
    Thriller,
    HistoricalFiction,
    Romance,
    GraphicNovel,
    ShortStory,
    YoungAdult,
    Children,
    Autobiography,
    Biography,
    FoodAndDrink,
    ArtAndPhotography,
    SelfHelp,
    History,
    Travel,
    TrueCrime,
    Humor,
    Essays,
    ReligionAndSpirituality,
}
