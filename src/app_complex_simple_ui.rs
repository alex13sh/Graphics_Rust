use iced::{
    Application, executor, Command, window::Mode, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
} 

mod ui;

use modbus::{ModbusValues, Device};
use std::collections::HashMap;
use std::collections::BTreeMap;

pub struct App {
    ui: UI,
    has_exit: bool,
    logic: meln_logic::init::Complect,
    
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    
}



#[derive(Default)]
struct UI {
    pb_exit: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonExit,
    
    TopHalfComplectUI(half_complect::Message),
    LowHalfComplectUI(half_complect::Message),
    
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let logic = meln_logic::init::Complect::new();

        let values_1 = logic.get_values().get_values_by_name_contains(&["М1"]);
        let values_2 = logic.get_values().get_values_by_name_contains(&["М2"]);
        
        (App {
            ui: Default::default(),
            has_exit: false,
            
            low: HalfComplect::new(values_1, logic.invertor_1.clone()),
            top: HalfComplect::new(values_2, logic.invertor_2.clone()),
            klapans: ui::Klapans::new(logic.digit_o.device().values_map()
                .get_values_by_name_starts(&["Клапан 24В", "Клапан 2", "Насос"])),
            dozator: ui::Dozator::new(logic.digit_o.device().values_map().clone()),
        
            logic: logic,
        },
        Command::none())
    }
    
    fn title(&self) -> String {
        String::from("Complect - Iced")
    }
    fn mode(&self) -> Mode {
        Mode::Fullscreen
    }
    fn should_exit(&self) -> bool {
        self.has_exit
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         Text::new("Complex View").into()
        self.low.view()
            .map(Message::LowHalfComplectUI)
            .into()
    }
}

use half_complect::HalfComplect;
mod half_complect {
    use super::*;
    
    pub struct HalfComplect {
        invertor: ui::Invertor,
        values: ModbusValues,
        
        values_list: Vec<ui::ValuesList>,
    }

    #[derive(Default)]
    struct UI {
    
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        InvertorUI(ui::invertor::Message),
        
    }

    impl HalfComplect {
        pub fn new(values: ModbusValues, invertor: modbus::Invertor) -> Self {
//             dbg!(values.keys());
            let values: HashMap<_,_> = values.iter()
                .filter(|(k,_)| k.matches("/").count()<=1)
                .map(|(k,v)| (k.clone(), v.clone()))
                .collect();
            let values = ModbusValues::from(values);
            dbg!(values.keys());
            
            HalfComplect {
                invertor: ui::Invertor::new(invertor),
                
                values_list: ui::make_value_lists_start(&values, map!{BTreeMap,
                    "Температуры" => [
                        "Температура статора",
                        "Температура ротора Пирометр",
                        "Температура масла на выходе",
                        "Температура подшипника",
                    ]
                }),
                values: values,
            }
        }
        
        pub fn view(&self) -> Element<Message> {
//             Text::new("Half Complect View").into()
            let list_value = self.values_list.iter()
                .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
                
            list_value.into()
        }
    }
}
