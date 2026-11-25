use leocard_ai::{DevicePreference, Network, NetworkBuilder, TensorMap};

pub(super) fn build(tensors: &TensorMap, device: DevicePreference) -> Result<Network, String> {
    tensors
        .get("stem.weight")
        .ok_or("麻将模型缺少 stem")?
        .expect_shape(&[128, 38, 3, 3])?;
    tensors
        .get("meta_blocks")
        .ok_or("麻将模型缺少网络层数")?
        .expect_shape(&[1])?;
    if tensors["meta_blocks"].values[0] != 40.0 {
        return Err("不支持的麻将模型层数".into());
    }
    tensors
        .get("foot.3.weight")
        .ok_or("麻将模型缺少策略头")?
        .expect_shape(&[235, 512])?;
    let mut builder = NetworkBuilder::new(vec![38, 4, 9]);
    let mut x = builder.conv3x3(0, tensors, "stem", true)?;
    for i in 0..40 {
        let y = builder.conv3x3(x, tensors, &format!("body.{i}.c1"), true)?;
        let y = builder.conv3x3(y, tensors, &format!("body.{i}.c2"), false)?;
        x = builder.add_relu(x, y)?;
    }
    x = builder.linear(x, tensors, "foot.1", true)?;
    x = builder.linear(x, tensors, "foot.3", false)?;
    builder.finish(x, device)
}
