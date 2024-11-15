## Extra Steps

IAM Policy via TF

```
eksctl utils associate-iam-oidc-provider --cluster YOUR_CLUSTER_NAME --approve
```

```
eksctl create iamserviceaccount \    
--cluster=YOUR_CLUSTER_NAME \  
--namespace=kube-system \  
--name=aws-load-balancer-controller \  
--attach-policy-arn=arn:aws:iam::<AWS_ACCOUNT_ID>:policy/AWSLoadBalancerControllerIAMPolicy \  
--override-existing-serviceaccounts \  
--approve
```

install controller via helm

```
helm repo add eks https://aws.github.io/eks-charts
```

```
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \      
--set clusterName=YOUR_CLUSTER_NAME \  
--set serviceAccount.create=false \  
--set region=YOUR_REGION_CODE \  
--set vpcId=<VPC_ID> \  
--set serviceAccount.name=aws-load-balancer-controller \  
-n kube-system
```