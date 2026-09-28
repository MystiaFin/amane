use std::cell::{Cell, RefCell};
use std::rc::Rc;

use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::subscribe::InterestMaskSet;
use libpulse_binding::context::{Context, FlagSet, State};
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::operation::{self, Operation};
use libpulse_binding::volume::{ChannelVolumes, Volume};

// pulse resolves this to whatever output is the default right now
const DEFAULT_SINK: &str = "@DEFAULT_SINK@";

#[derive(Default, Clone, Copy)]
pub struct Sink {
    // 0 to 100
    pub volume: u8,

    pub muted: bool,
}

// what pulse reports for the default output, before converting to percent
#[derive(Clone, Copy)]
struct Levels {
    volumes: ChannelVolumes,
    muted: bool,
}

thread_local! {
    // pulse objects can't move between threads, so each thread opens its own
    static CONNECTION: RefCell<Connection> = RefCell::new(Connection::open());
}

pub fn read() -> Sink {
    CONNECTION.with_borrow_mut(|connection| {
        let Some(levels) = connection.default_sink() else {
            return Sink::default();
        };

        Sink {
            volume: percent(levels.volumes.avg()),
            muted: levels.muted,
        }
    })
}

pub fn set_volume(volume: u8) {
    CONNECTION.with_borrow_mut(|connection| {
        let Some(levels) = connection.default_sink() else {
            return;
        };

        // every channel gets the same level, like a single volume slider does
        let mut volumes = levels.volumes;
        let channels = volumes.len();

        volumes.set(channels, level(volume));

        let operation = connection
            .context
            .introspect()
            .set_sink_volume_by_name(DEFAULT_SINK, &volumes, None);

        connection.wait(operation);
    });
}

pub fn set_muted(muted: bool) {
    CONNECTION.with_borrow_mut(|connection| {
        let operation = connection
            .context
            .introspect()
            .set_sink_mute_by_name(DEFAULT_SINK, muted, None);

        connection.wait(operation);
    });
}

/*
 * calls changed after every change to an output or to the
 * default choice, and never returns; it keeps its own
 * connection, so changed can still use read()
 */
pub fn watch(mut changed: impl FnMut()) {
    let mut connection = Connection::open();

    let dirty = Rc::new(Cell::new(false));
    let marked = Rc::clone(&dirty);

    /*
     * pulse can't answer questions from inside its own callback,
     * so the callback only marks the change and the loop reacts
     */
    connection
        .context
        .set_subscribe_callback(Some(Box::new(move |_, _, _| marked.set(true))));

    let interests = InterestMaskSet::SINK | InterestMaskSet::SERVER;

    connection.context.subscribe(interests, |_| {});

    loop {
        connection.iterate();

        if dirty.replace(false) {
            changed();
        }
    }
}

struct Connection {
    mainloop: Mainloop,
    context: Context,
}

impl Connection {
    fn open() -> Self {
        let mainloop = Mainloop::new().expect("failed to create pulse mainloop");

        let mut context = Context::new(&mainloop, "amane").expect("failed to create pulse context");

        context
            .connect(None, FlagSet::NOFLAGS, None)
            .expect("failed to connect to pulse");

        let mut connection = Self { mainloop, context };

        loop {
            connection.iterate();

            match connection.context.get_state() {
                State::Ready => return connection,
                State::Failed | State::Terminated => panic!("failed to connect to pulse"),
                _ => {}
            }
        }
    }

    // none when there is no output at all
    fn default_sink(&mut self) -> Option<Levels> {
        let found = Rc::new(Cell::new(None));
        let slot = Rc::clone(&found);

        let operation = self
            .context
            .introspect()
            .get_sink_info_by_name(DEFAULT_SINK, move |result| {
                let ListResult::Item(sink) = result else {
                    return;
                };

                slot.set(Some(Levels {
                    volumes: sink.volume,
                    muted: sink.mute,
                }));
            });

        self.wait(operation);

        found.take()
    }

    // blocks until pulse has answered
    fn wait<F: ?Sized>(&mut self, operation: Operation<F>) {
        while operation.get_state() == operation::State::Running {
            self.iterate();
        }
    }

    fn iterate(&mut self) {
        let IterateResult::Success(_) = self.mainloop.iterate(true) else {
            panic!("failed to run pulse mainloop");
        };
    }
}

// pulse counts volume with 100% at Volume::NORMAL
fn percent(volume: Volume) -> u8 {
    let level = u64::from(volume.0);
    let normal = u64::from(Volume::NORMAL.0);

    let rounded = (level * 100 + normal / 2) / normal;

    rounded.min(100) as u8
}

fn level(percent: u8) -> Volume {
    let normal = Volume::NORMAL.0;

    Volume(u32::from(percent) * normal / 100)
}
