use sensor_kit::sampling::Sampler;
use sensor_kit::Sensor;

fn sensor() -> Sensor {
    Sensor {
        id: 7,
        address: 0x10,
        scale: 0.5,
        offset: 1.0,
        connected: true,
    }
}

#[test]
fn samples_once_per_period() {
    let mut sampler = Sampler::new(sensor(), 10, 100);

    assert_eq!(sampler.poll(0), Some(9.0));
    assert_eq!(sampler.poll(5), None);
    assert_eq!(sampler.poll(10), Some(9.0));

    assert_eq!(sampler.reading(false, false), Some(9.0));
    assert_eq!(sampler.reading(true, false), Some(16.0));
    assert_eq!(sampler.reading(false, true), Some(9.0));
    assert_eq!(sampler.into_sensor().address, 0x10);
}
