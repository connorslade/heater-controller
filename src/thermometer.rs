use embassy_futures::block_on;
use embassy_stm32::{
    Peri,
    adc::{Adc, AdcChannel, AnyAdcChannel, Resolution, SampleTime, VREF_CALIB_MV},
    peripherals::{ADC1, DMA1_CH1, PA3},
};
use micromath::F32Ext;

use crate::{
    Irqs,
    config::{system::TEMP_SAMPLES, thermister::*},
    misc::ring_buffer::RingBuffer,
};

const VREFINT_CAL: *const u16 = 0x1FFF756A as _;

pub struct Thermometer<'a> {
    adc: Adc<'a, ADC1>,
    dma: Peri<'a, DMA1_CH1>,
    vref: AnyAdcChannel<'a, ADC1>,

    liquid: AnyAdcChannel<'a, ADC1>,
    liquid_buffer: RingBuffer<f32, TEMP_SAMPLES>,
}

impl<'a> Thermometer<'a> {
    pub fn new(adc: Peri<'a, ADC1>, dma: Peri<'a, DMA1_CH1>, liquid: Peri<'a, PA3>) -> Self {
        let adc = Adc::new(adc, Resolution::BITS12);
        Self {
            vref: adc.enable_vrefint().degrade_adc(),
            adc,
            dma,

            liquid: liquid.degrade_adc(),
            liquid_buffer: RingBuffer::new(),
        }
    }

    pub fn sample(&mut self) -> f32 {
        let mut readings = [0; 2];

        let sequence = [
            (&mut self.liquid, SampleTime::CYCLES640_5),
            (&mut self.vref, SampleTime::CYCLES640_5),
        ];
        let future = self.adc.read(
            self.dma.reborrow(),
            Irqs,
            sequence.into_iter(),
            &mut readings,
        );
        block_on(future);

        let [input, vref] = readings;
        let vdd = (VREF_CALIB_MV as f32 * unsafe { *VREFINT_CAL } as f32) / vref as f32;
        let input = (input as f32 / 4096.0) * vdd;
        let r = R2 * (vdd / input - 1.0);
        let t = thermistor_temp(r);

        self.liquid_buffer.push(t);
        self.liquid_buffer.avg()
    }
}

fn thermistor_temp(resistance: f32) -> f32 {
    1.0 / (1.0 / T0 + (resistance / R0).ln() / BETA) - 273.15
}
