## Manual steps

1. Create cluster:

   ```
   eksctl create cluster -f cluster.yml
   ```
2. Associate OIDC provider

   ```
   eksctl utils associate-iam-oidc-provider --cluster YOUR_CLUSTER_NAME --approve
   ```
3. Create service account for albc.
    ```
    eksctl create iamserviceaccount \    
    --cluster=hydra-doom-dev-cluster \  
    --namespace=kube-system \  
    --name=aws-load-balancer-controller \  
    --attach-policy-arn=arn:aws:iam::YOUR_ACCOUNT_ID:policy/AWSLoadBalancerControllerIAMPolicy \  
    --override-existing-serviceaccounts \  
    --approve
    ```
4. Install ALBC via helm.

   ```
   helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
       --set clusterName=YOUR_CLUSTER_NAME \
       --set serviceAccount.create=false \
       --set region=YOUR_REGION_CODE \
       --set vpcId=$(aws eks describe-cluster --name $YOUR_CLUSTER_NAME --region YOUR_REGION_CODE | jq -r '.cluster.resourcesVpcConfig.vpcId') \
       --set serviceAccount.name=aws-load-balancer-controller \
       -n kube-system
   ```
