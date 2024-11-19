## Manual steps

### Prerequisites

1. Create ALBC IAM policy (this should be created once for the whole account,
   not per cluster).

### Cluster creation steps

1. Create cluster. You must modify the `.metadata.name`, `.metadata.region` and
   `.managedNodeGroups[].availabilityZones` values accordingly.

   ```
   eksctl create cluster -f cluster.yml
   ```
2. Create service account for albc (ALBC is not an addon, so service account
   must be created and linked).
    ```
    eksctl create iamserviceaccount \    
    --cluster=hydra-doom-dev-cluster \  
    --namespace=kube-system \  
    --name=aws-load-balancer-controller \  
    --attach-policy-arn=arn:aws:iam::YOUR_ACCOUNT_ID:policy/AWSLoadBalancerControllerIAMPolicy \  
    --override-existing-serviceaccounts \  
    --approve
    ```
3. Install ALBC via helm.

   ```
   helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
       --set clusterName=YOUR_CLUSTER_NAME \
       --set serviceAccount.create=false \
       --set region=YOUR_REGION_CODE \
       --set vpcId=$(aws eks describe-cluster --name $YOUR_CLUSTER_NAME --region YOUR_REGION_CODE | jq -r '.cluster.resourcesVpcConfig.vpcId') \
       --set serviceAccount.name=aws-load-balancer-controller \
       -n kube-system
   ```
4. Create SSL cert on the corresponding region using AWS Cert Manager (you will
   need the ARN to set up the ingress controller on `stage1`).
