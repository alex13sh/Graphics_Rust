
impl Default for InvertorError {
    fn default() -> Self {
        InvertorError::None
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum InvertorError {
    #[error("Аварий не зафиксировано")]
    None = 0,
    #[error("Перегрузка по току во время разгона (ocA)")]
    ocA = 1,
    #[error("Перегрузка по току во время замедления (ocd)")]
    ocd = 2,
    #[error("Перегрузка по току в установившемся режиме (ocn)")]
    ocn = 3,
    #[error("Замыкание на землю (GFF)")]
    GFF = 4,
    #[error("Короткое замыкание IGBT-модуля (occ)")]
    occ = 5,
    #[error("Перегрузка по току в режиме СТОП (ocS)")]
    ocS = 6,
    #[error("Перенапряжение во время разгона (ovA)")]
    ovA = 7,
    #[error("Перенапряжение во время замедления (ovd)")]
    ovd = 8,
    
    #[error("Перенапряжение в установившемся режиме (ovn)")]
    ovn = 9,
    #[error("Перенапряжение в режиме СТОП (ovS)")]
    ovS = 10,
    #[error("Низкое напряжение во время разгона (LvA)")]
    LvA = 11,
    #[error("Низкое напряжение во время замедления (Lvd)")]
    Lvd = 12,
    #[error("Низкое напряжение в установившемся режиме (Lvn)")]
    Lvn = 13,
    #[error("Низкое напряжение в режиме СТОП (LvS)")]
    LvS = 14,
    #[error("Отсутствие входной фазы (OrP)")]
    OrP = 15,
    #[error("Перегрев IGBT-модуля (oH1)")]
    oH1 = 16,
    #[error("Перегрев радиатора (oH2) (от 30кВт)")]
    oH2 = 17,
    #[error("tH1o (TH1: отказ термодатчика IGBT)")]
    TH1 = 18,
    #[error("tH2o (TH2: отказ термодатчика радиатора)")]
    TH2 = 19,
    
    #[error("Перегрузка привода по току (oL)")]
    oL = 21,
    #[error("Электронная тепловая защита двигателя 1 (EoL1)")]
    EoL1 = 22,
    #[error("Электронная тепловая защита двигателя 2 (EoL2)")]
    EoL2 = 23,
    #[error("Перегрев двигателя, зафиксированный датчиком PTC (oH3) (PTC)")]
    oH3 = 24,
    
    #[error("Превышение момента 1 (ot1)")]
    ot1 = 26,
    #[error("Превышение момента 2 (ot2)")]
    ot2 = 27,
    #[error("Низкий уровень тока (uC)")]
    uC = 28,
    #[error("Ошибка выхода за границы исходного положения (LMIT)")]
    LMIT = 29,
    #[error("Ошибка записи в EEPROM (cF1)")]
    cF1 = 30,
    #[error("Ошибка чтения EEPROM (cF2)")]
    cF2 = 31,
    
    #[error("Ошибка определения тока U-фазы (cd1)")]
    cd1 = 33,
    #[error("Ошибка определения тока V-фазы (cd2)")]
    cd2 = 34,
    #[error("Ошибка определения тока W-фазы (cd3)")]
    cd3 = 35,
    #[error("Аппаратная ошибка рампы тока (Hd0)")]
    Hd0 = 36,
    #[error("Аппаратная ошибка, перегрузка по току (Hd1)")]
    Hd1 = 37,
    #[error("Аппаратная ошибка, перенапряжение (Hd2)")]
    Hd2 = 38,
    #[error("Аппаратная ошибка, короткое замыкание IGBT-модуля (Hd3)")]
    Hd3 = 39,
    #[error("Ошибка автотестирования двигателя (AuE)")]
    AuE = 40,
    #[error("Потеря обратной связи ПИД (AFE)")]
    AFE = 41,
    #[error("Ошибка обратной связи PG (PGF1)")]
    PGF1 = 42,
    #[error("Потеря обратной связи PG (PGF2)")]
    PGF2 = 43,
    #[error("Срыв обратной связи PG (PGF3)")]
    PGF3 = 44,
    #[error("Ошибка по скольжению PG (PGF4)")]
    PGF4 = 45,
    #[error("Ошибка задания PG (PGr1)")]
    PGr1 = 46,
    #[error("Ошибка задания PG (PGr2)")]
    PGr2 = 47,
    #[error("Потеря сигнала на входе ACI (ACE)")]
    ACE = 48,
     #[error("Внешнее аварийное отключение (EF)")]
    EF = 49,
     #[error("Внешний аварийный стоп (EF1)")]
    EF1 = 50,
     #[error("Пауза в работе (bb)")]
    bb = 51,
     #[error("Ошибка ввода пароля (PcodE)")]
    PcodE = 52,
     #[error("Коммуникационная ошибка (cE1)")]
    cE1 = 54,
    #[error("Коммуникационная ошибка (cE2)")]
    cE2 = 55,
    #[error("Коммуникационная ошибка (cE3)")]
    cE3 = 56,
    #[error("Коммуникационная ошибка (cE4)")]
    cE4 = 57,
    #[error("Превышено время ожидания коммуникации (cE10)")]
    cE10 = 58,
    #[error("Превышение времени при связи с пультом управления (cP10)")]
    cP10 = 59,
    #[error("Сбой в работе тормозного резистора (bF)")]
    bF = 60,
    #[error("Ошибка переключения Y /Δ (ydc)")]
    ydc = 61,
    #[error("Ошибка управляемого торможения за счет запасенной энергии (dEb)")]
    dEb = 62,
    #[error("Ошибка скольжения (oSL)")]
    oSL = 63,
    #[error("Ошибка переключения магнитного контактора (ryF)")]
    ryF = 64,
    #[error("Ошибка PG карты (PGF5)")]
    PGF5 = 65,

    #[error("Sensorless estimated speed have wrong direction")]
    e68 = 68,
    #[error("Sensorless estimated speed is over speed")]
    e69 = 69,
    #[error("Sensorless estimated speed deviated")]
    e70 = 70,
    #[error("Watchdog")]
    wd = 71,
    #[error("Channel 1 (STO1–SCM1) safety loop error (STL1)")]
    STL1 = 72,

    
    #[error("Ошибка функции безопасного останова (S1)")]
    S1 = 73,

    #[error("External brake error")]
    e75 = 75,
    #[error("Safe Torque Off (STO)")]
    STO = 76,
    #[error("Channel 2 (STO2–SCM2) safety loop error (STL2)")]
    STL2 = 77,
    #[error("Internal loop error (STL3)")]
    STL3 = 78,
    
    #[error("Uocc Превышение тока U-фазы (мониторинг начинается при нажатии кнопки RUN, программная защита)")]
    Uocc = 79,
    #[error("Vocc Превышение тока V-фазы (мониторинг начинается при нажатии кнопки RUN, программная защита)")]
    Vocc = 80,
    #[error("Wocc Превышение тока W-фазы (мониторинг начинается при нажатии кнопки RUN, программная защита)")]
    Wocc = 81,
    #[error("OPHL обрыв выходной фазы U")]
    OPHL_U = 82,
    #[error("OPHL обрыв выходной фазы V")]
    OPHL_V = 83,
    #[error("OPHL обрыв выходной фазы W")]
    OPHL_W = 84,

    #[error("PG-02U ABZ hardware disconnection")]
    e85 = 85,
    #[error("PG-02U UVW hardware disconnection")]
    e86 = 86,
    #[error("oL3 Low frequency overload protection")]
    oL3 = 87,
    #[error("RoPd initial rotor position detection error")]
    RoPd = 89,
    #[error("Inner PLC function is forced to stop")]
    PLC = 90,
    #[error("CPU error")]
    CPU = 93,

    
    #[error("CGdE Превышение времени сторожевого запроса CANopen")]
    CGdE = 101,
    #[error("CHbE Превышено время ожидания контрольных сообщений (тактирования) CANopen")]
    CHbE = 102,
    #[error("CSyE Ошибка синхронизации CANopen")]
    CSyE = 103,
    #[error("CbFE Шина CANopen не доступна")]
    CbFE = 104,
    #[error("CIdE Ошибка CANopen индекса")]
    CIdE = 105,
    #[error("CAdE Ошибка адреса ведомой станции CANopen")]
    CAdE = 106,
    #[error("CFrE Слишком длинный CANopen индекс")]
    CFrE = 107,
    
    // #[error("")]
}
