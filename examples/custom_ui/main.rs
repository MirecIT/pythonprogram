//! Custom UI example

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{
        plugins::RenderToWindow,
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, TexCoord},
        },
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, ToNativeWidget, UiBundle, UiCreator, UiTransformData, UiWidget},
    utils::{application_root_dir, scene::BasicScenePrefab},
};
use serde::Deserialize;

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

#[derive(Clone, Deserialize)]
enum CustomUi {
    // Example widget which repeats its `item`
    Repeat {
        x_move: f32,
        y_move: f32,
        count: usize,
        item: UiWidget<CustomUi>,
    },
}

impl ToNativeWidget for CustomUi {
    type PrefabData = ();
    fn to_native_widget(self, _: ()) -> (UiWidget<CustomUi>, Self::PrefabData) {
        match self {
            CustomUi::Repeat {
                count,
                item,
                x_move,
                y_move,
            } => {
                #[allow(clippy::redundant_closure)] // Inference fails on Default::default otherwise
                let transform = item
                    .transform()
                    .cloned()
                    .unwrap_or_else(|| Default::default());
                let mut pos = (0., 0., 100.);
                let children = std::iter::repeat(item)
                    .map(|widget| {
                        let widget = UiWidget::Container {
                            background: None,
                            transform: UiTransformData::default()
                                .with_position(pos.0, pos.1, pos.2),
                            children: vec![widget],
                        };
                        pos.0 += x_move;
                        pos.1 += y_move;
                        widget
                    })
                    .take(count)
                    .collect();
                let widget = UiWidget::Container {
                    background: None,
                    transform,
                    children,
                };
                (widget, ())
            }
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData { world, .. } = data;
        // initialize the scene with an object, a light and a camera.
        let handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, ())
        });
        world.push((handle,));

        // Load custom UI prefab
        world.exec(|mut creator: UiCreator<'_, CustomUi>| {
            creator.create("ui/custom.ron", ());
        });
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets");

    let mut game_data = DispatcherBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .add_bundle(TransformBundle::new())?
        .add_bundle(InputBundle::new())?
        .add_bundle(UiBundle::<StringBindings, CustomUi>::new())?
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderUi::default()),
        )?;

    let game = Application::build(assets_dir, Example)?.build(game_data)?;
    game.run();
    Ok(())
}
