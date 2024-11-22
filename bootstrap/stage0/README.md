## Manual steps

### Prerequisites

1. Create ALBC IAM policy (this should be created once for the whole account,
   not per cluster).

   ```
   aws iam create-policy \
     --policy-name AWSLoadBalancerControllerIAMPolicy \
     --policy-document file://albc_iam_policy.json
   ```

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
    --cluster=YOUR_CLUSTER_NAME \  
    --namespace=kube-system \  
    --name=aws-load-balancer-controller \  
    --attach-policy-arn=arn:aws:iam::YOUR_ACCOUNT_ID:policy/AWSLoadBalancerControllerIAMPolicy \  
    --override-existing-serviceaccounts \  
    --approve
    ```
3. Create service account for EBS:
   ```
   eksctl create iamserviceaccount \
        --name ebs-csi-controller-sa \
        --namespace kube-system \
        --cluster YOUR_CLUSTER_NAME \
        --attach-policy-arn arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy \
        --override-existing-serviceaccounts \
        --approve
   ```
4. Create service account for Hydra Doom Nodes (tipically
   `HYDRA_DOOM_NODE_SERVICE_ACCOUNT=hydra-doom-node`,
   `HYDRA_DOOM_NAMESPACE=hydra-doom`):
   ```
   eksctl create iamserviceaccount \
        --name HYDRA_DOOM_NODE_SERVICE_ACCOUNT \
        --namespace HYDRA_DOOM_NAMESPACE \
        --cluster YOUR_CLUSTER_NAME \
        --attach-policy-arn arn:aws:iam::509399595051:policy/hydra-doom-kinesis-writer \
        --override-existing-serviceaccounts \
        --approve
   ```
5. Install ALBC via helm.

   ```
   helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
       --set clusterName=YOUR_CLUSTER_NAME \
       --set serviceAccount.create=false \
       --set region=YOUR_REGION_CODE \
       --set vpcId=$(aws eks describe-cluster --name $YOUR_CLUSTER_NAME --region YOUR_REGION_CODE | jq -r '.cluster.resourcesVpcConfig.vpcId') \
       --set serviceAccount.name=aws-load-balancer-controller \
       -n kube-system
   ```
6. Create SSL cert on the corresponding region using AWS Cert Manager (you will
   need the ARN to set up the ingress controller on `stage1`).
