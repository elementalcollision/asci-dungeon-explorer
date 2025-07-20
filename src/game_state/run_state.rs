#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: usize },
    MainMenu,
    SaveGame,
    LoadGame,
    GameOver,
    LevelUp,
    Examine,
    QuitGame,
    
    // Character creation states
    CharacterCreation,
    CharacterName,
    CharacterClass,
    CharacterBackground,
    CharacterAttributes,
    CharacterEquipment,
    CharacterConfirm,
}