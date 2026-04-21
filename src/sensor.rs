use rp2040_hal::{fugit::MicrosDurationU32, timer::Alarm};

const ARM_VALUE: MicrosDurationU32 = MicrosDurationU32::minutes(1);

pub struct SensorReadingAverager<A> {
    alarm: A,

    sum_tempe: f32,
    sum_humid: f32,
    sum_bpres: f32,
    sum_pm1_0: f32,
    sum_pm2_5: f32,
    sum_pm_10: f32,

    readings: f32,
}

#[derive(Debug)]
pub struct SensorReadings {
    pub tempe: f32,
    pub humid: f32,
    pub bpres: f32,
    pub pm1_0: f32,
    pub pm2_5: f32,
    pub pm_10: f32,
}

impl<A: Alarm> SensorReadingAverager<A> {
    pub fn new(alarm: A) -> Self {
        Self {
            alarm,
            sum_tempe: 0.0,
            sum_humid: 0.0,
            sum_bpres: 0.0,
            sum_pm1_0: 0.0,
            sum_pm2_5: 0.0,
            sum_pm_10: 0.0,
            readings: 0.0,
        }
    }

    pub fn arm(&mut self) {
        self.alarm.schedule(ARM_VALUE).ok();
    }

    pub fn add_reading(&mut self, readings: SensorReadings) {
        self.sum_tempe += readings.tempe;
        self.sum_humid += readings.humid;
        self.sum_bpres += readings.bpres;
        self.sum_pm1_0 += readings.pm1_0;
        self.sum_pm2_5 += readings.pm2_5;
        self.sum_pm_10 += readings.pm_10;
        self.readings += 1.0;
    }

    pub fn report(&mut self) -> Option<SensorReadings> {
        if !self.alarm.finished() {
            return None;
        }
        self.alarm.cancel().ok();
        self.arm();

        let tempe = self.sum_tempe / self.readings;
        let humid = self.sum_humid / self.readings;
        let bpres = self.sum_bpres / self.readings;
        let pm1_0 = self.sum_pm1_0 / self.readings;
        let pm2_5 = self.sum_pm2_5 / self.readings;
        let pm_10 = self.sum_pm_10 / self.readings;

        self.sum_tempe = 0.0;
        self.sum_humid = 0.0;
        self.sum_bpres = 0.0;
        self.sum_pm1_0 = 0.0;
        self.sum_pm2_5 = 0.0;
        self.sum_pm_10 = 0.0;
        self.readings = 0.0;

        Some(SensorReadings {
            tempe,
            humid,
            bpres,
            pm1_0,
            pm2_5,
            pm_10,
        })
    }
}
