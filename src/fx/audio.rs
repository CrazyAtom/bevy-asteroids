use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sfx {
    Fire,
    Explosion,
    UfoFire,
    Pickup,
    Special,
    Hyperspace,
    GameOver,
}

impl Sfx {
    fn file(self) -> &'static str {
        match self {
            Sfx::Fire => "sounds/fire.wav",
            Sfx::Explosion => "sounds/explosion.wav",
            Sfx::UfoFire => "sounds/ufo_fire.wav",
            Sfx::Pickup => "sounds/pickup.wav",
            Sfx::Special => "sounds/special.wav",
            Sfx::Hyperspace => "sounds/hyperspace.wav",
            Sfx::GameOver => "sounds/game_over.wav",
        }
    }
}

#[derive(Message)]
pub struct SfxEvent(pub Sfx);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SfxEvent>().add_systems(Update, play_sfx);
    }
}

fn play_sfx(mut commands: Commands, asset_server: Res<AssetServer>, mut events: MessageReader<SfxEvent>) {
    for SfxEvent(sfx) in events.read() {
        commands.spawn((
            AudioPlayer::new(asset_server.load(sfx.file())),
            PlaybackSettings::DESPAWN,
        ));
    }
}
