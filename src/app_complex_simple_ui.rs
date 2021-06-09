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
    
    DozatorUI(ui::dozator::Message),
    
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
            dozator: ui::Dozator::new(logic.dozator.clone()),
        
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
    fn scale_factor(&self) -> f64 {0.8}

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        Message::ButtonExit => {},
        Message::LowHalfComplectUI(m) => self.low.update(m),
        Message::TopHalfComplectUI(m) => self.top.update(m),
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, vec![self.logic.digit_o.device().clone()])
                .map(Message::DozatorUI);
            self.logic.update_new_values();
            return res;
        },
        }
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         Text::new("Complex View").into()
        let low = self.low.view()
            .map(Message::LowHalfComplectUI);
        let top = self.top.view()
            .map(Message::TopHalfComplectUI);
            
        let half_2 = Row::new()
            .spacing(20)
            .push(low)
            .push(top);
        let dozator = self.dozator.view().map(Message::DozatorUI);
        let col = Column::new()
            .spacing(10)
            .push(half_2)
            .push(dozator);
        col.into()
//         Container::new(col)
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .padding(10)
//             .center_x()
//             .center_y()
//             .into()
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
//         pub fn new_by_name(values: ModbusValues
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
        
        pub fn update(&mut self, _message: Message) {
        
        }
        
        pub fn view(&mut self) -> Element<Message> {
//             Text::new("Half Complect View").into()
            let list_value = self.values_list.iter()
                .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
            let inv = self.invertor.view().map(Message::InvertorUI);
            let list_value = list_value.push(inv);
            list_value.into()
        }
    }
}
