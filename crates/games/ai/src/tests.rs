use crate::{DevicePreference, Network, NetworkBuilder, Tensor, TensorArchive, TensorMap};

fn network(device: DevicePreference) -> Network {
    let mut tensors = TensorMap::new();
    for name in ["first", "second"] {
        tensors.insert(
            format!("{name}.weight"),
            Tensor::new(
                vec![2, 2, 3, 3],
                (0..36).map(|n| (n as f32 - 17.0) / 100.0).collect(),
            )
            .unwrap(),
        );
        tensors.insert(
            format!("{name}.bias"),
            Tensor::new(vec![2], vec![0.1, -0.2]).unwrap(),
        );
    }
    tensors.insert(
        "dense.weight".into(),
        Tensor::new(
            vec![3, 12],
            (0..36).map(|n| (n as f32 - 18.0) / 10.0).collect(),
        )
        .unwrap(),
    );
    tensors.insert(
        "dense.bias".into(),
        Tensor::new(vec![3], vec![1.0, 2.0, 3.0]).unwrap(),
    );
    let mut builder = NetworkBuilder::new(vec![2, 2, 3]);
    let first = builder.conv3x3(0, &tensors, "first", true).unwrap();
    let second = builder.conv3x3(first, &tensors, "second", false).unwrap();
    let sum = builder.add_relu(first, second).unwrap();
    let output = builder.linear(sum, &tensors, "dense", false).unwrap();
    builder.finish(output, device).unwrap()
}

#[test]
fn converted_archive_preserves_weights_and_rejects_truncation() {
    let tensors = TensorMap::from([(
        "weight".into(),
        Tensor::new(vec![2], vec![0.25, -3.5]).unwrap(),
    )]);
    let bytes = TensorArchive::encode(&tensors).unwrap();
    let decoded = TensorArchive::decode(&bytes).unwrap();
    assert_eq!(decoded["weight"].values.as_ref(), &[0.25, -3.5]);
    assert!(TensorArchive::decode(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn residual_convolution_and_linear_support_independent_batches() {
    let network = network(DevicePreference::Cpu);
    let first: Vec<_> = (0..12).map(|n| n as f32 / 10.0).collect();
    let second: Vec<_> = first.iter().map(|n| -n).collect();
    let batch: Vec<_> = first.iter().chain(&second).copied().collect();
    let actual = network.run(&batch, 2).unwrap();
    assert_eq!(&actual[..3], network.run(&first, 1).unwrap());
    assert_eq!(&actual[3..], network.run(&second, 1).unwrap());
    assert_ne!(actual[..3], actual[3..]);
}

#[test]
#[ignore = "requires a hardware GPU; run explicitly on a GPU host"]
fn gpu_executes_the_same_residual_network_as_cpu() {
    let input: Vec<_> = (0..24).map(|n| (n as f32 - 12.0) / 10.0).collect();
    let expected = network(DevicePreference::Cpu).run(&input, 2).unwrap();
    let actual = network(DevicePreference::Gpu).run(&input, 2).unwrap();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
    }
}
