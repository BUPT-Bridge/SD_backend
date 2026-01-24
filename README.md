# SD Backend
## 开发指南
### 微信模拟接口
首先运行`debug_utils/mock_wx.py`

然后在`.env`文件中添加以下内容来模拟微信接口：

```
SERVER_WX_BASEURL=http://localhost:5000
```

### Reqable序列化指南
安装protoc并添加到环境变量

如果是windows,修改`debug_utils/reqable_template.py`第43行的`protoc`为`protoc.exe`

根据接口的不同配置`PROTO_STRING`常量

在encode和decode的时候选择合适的message名称即可

### 新接口的编写
所有接口和数据库的连接都写在`router`下面即可

按照路由的结构组织文件

在`lib.rs`中注册路由即可

> protobuf在`interface_types/proto`下面写,并在`mod.rs`中引入
