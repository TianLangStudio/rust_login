[English](README.md) | 简体中文 
# 使用Rust实现用户登录
```
Web第一步用户先登录 
``` 

## 更新记录

1. [让客户端可以使用Json格式的数据提交用户名和密码](https://blog.csdn.net/tianlangstudio/article/details/106169242) 

2. 根据密码是否跟用户名一致返回成功或失败　
> commit id   541075e 

3. 统一返回结果类型 

4. 添加Session支持　
> 使用actix session中间件 

5. 使用Blake2生成签名信息 

6. 使用rustls提供https服务  

7. 使用config管理配置文件   

8. 使用log4rs管理日志  

9. 使用Diesel操作数据库  
