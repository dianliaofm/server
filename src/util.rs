use crate::entity::Episode;

pub fn init_log() {
    let _lg = flexi_logger::Logger::try_with_env_or_str("debug")
        .unwrap()
        .log_to_stdout()
        .start()
        .unwrap();
}

pub fn filter_time(timestamp: u64) -> impl Fn(&Episode) -> bool {
    move |e: &Episode| -> bool { e.timestamp > timestamp }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_test() {
        let t1 = 10u64;
        let eps = (8..13)
            .map(|x| Episode {
                timestamp: x,
                ..Default::default()
            })
            .filter(filter_time(t1))
            .collect::<Vec<Episode>>();
        assert_eq!(eps.len(), 2);
        assert_eq!(
            eps,
            vec![
                Episode {
                    timestamp: 11,

                    ..Default::default()
                },
                Episode {
                    timestamp: 12,

                    ..Default::default()
                }
            ]
        )
    }
}
